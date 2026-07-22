use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tauri::{AppHandle, Emitter, State};

use crate::copier::conflict::ConflictStrategy;
use crate::copier::parallel::FileCopier;
use crate::copier::progress::BackupProgressEmitter;
use crate::copier::{BackupFile, BackupResult, FailedFile};
use crate::database::models::BackupHistoryEntry;
use crate::database::queries;
use crate::database::Database;

fn create_backup_task(
  db: &Database,
  source_root: &str,
  dest_path: &str,
  total_files: u64,
  total_bytes: u64,
) -> Result<i64, String> {
  let conn = db.conn.lock().map_err(|e| e.to_string())?;
  conn
    .execute(
      "INSERT INTO backup_tasks (source_root, dest_path, total_files, total_bytes, backed_up_files, backed_up_bytes, failed_files, status, started_at)
       VALUES (?1, ?2, ?3, ?4, 0, 0, 0, 'running', datetime('now'))",
      rusqlite::params![source_root, dest_path, total_files, total_bytes],
    )
    .map_err(|e| e.to_string())?;
  let id = conn.last_insert_rowid();
  Ok(id)
}

fn update_backup_task(
  db: &Database,
  task_id: i64,
  backed_up_files: u64,
  backed_up_bytes: u64,
  failed_files: u64,
  status: &str,
  avg_speed: f64,
) {
  if let Ok(conn) = db.conn.lock() {
    let _ = conn.execute(
      "UPDATE backup_tasks SET backed_up_files = ?1, backed_up_bytes = ?2, failed_files = ?3,
       status = ?4, avg_speed = ?5, completed_at = datetime('now')
       WHERE id = ?6",
      rusqlite::params![backed_up_files, backed_up_bytes, failed_files, status, avg_speed, task_id],
    );
  }
}

pub struct BackupState {
  pub cancel: Arc<AtomicBool>,
  pub paused: Arc<AtomicBool>,
}

impl BackupState {
  pub fn new() -> Self {
    Self {
      cancel: Arc::new(AtomicBool::new(false)),
      paused: Arc::new(AtomicBool::new(false)),
    }
  }
}

#[derive(serde::Deserialize)]
pub struct StartBackupOptions {
  pub source_root: String,
  pub dest_path: String,
  pub files: Vec<BackupFile>,
  pub conflict_strategy: Option<String>,
  pub concurrency: Option<usize>,
}

#[tauri::command]
pub async fn start_backup(
  app: AppHandle,
  options: StartBackupOptions,
  db: State<'_, Database>,
  state: State<'_, BackupState>,
) -> Result<BackupResult, String> {
  // 重置状态
  state.cancel.store(false, Ordering::SeqCst);
  state.paused.store(false, Ordering::SeqCst);

  let dest_root = Path::new(&options.dest_path);
  if !dest_root.exists() {
    std::fs::create_dir_all(dest_root).map_err(|e| e.to_string())?;
  }

  let total_files = options.files.len() as u64;
  let total_bytes: u64 = options.files.iter().map(|f| f.file_size).sum();

  let concurrency = options.concurrency.unwrap_or(4).clamp(1, 16);
  let strategy = ConflictStrategy::from_str(
    options
      .conflict_strategy
      .as_deref()
      .unwrap_or("rename"),
  );

  let progress = BackupProgressEmitter::new(Some(Arc::new(app.clone())), total_files, total_bytes);

  progress.emit_log(&format!(
    "Starting backup: {} files, {} bytes, {} threads",
    total_files, total_bytes, concurrency
  ));

  let source_root = options
    .source_root
    .trim_end_matches(std::path::MAIN_SEPARATOR_STR)
    .to_string();

  // 创建备份任务记录
  let task_id = create_backup_task(&db, &source_root, &options.dest_path, total_files, total_bytes)
    .map_err(|e| format!("Failed to create backup task: {}", e))?;

  let copier = FileCopier::new(concurrency, strategy);
  let result = copier.run(
    options.files,
    dest_root,
    progress.clone(),
    state.cancel.clone(),
    state.paused.clone(),
  )?;

  // 更新备份任务
  let is_cancelled = state.cancel.load(Ordering::SeqCst);
  update_backup_task(
    &db,
    task_id,
    result.succeeded,
    result.copied_bytes,
    result.failed,
    if is_cancelled { "cancelled" } else { "completed" },
    result.avg_speed_mbps,
  );

  // 更新文件记录
  if let Err(e) = update_backup_records(&db, &source_root, &options.dest_path, &result) {
    progress.emit_log(&format!("Warning: failed to update database: {}", e));
  }

  progress.emit_log(&format!(
    "Backup complete: {} succeeded, {} failed, {:.2} MB/s avg",
    result.succeeded, result.failed, result.avg_speed_mbps
  ));

  app
    .emit("backup:complete", &result)
    .map_err(|e| e.to_string())?;

  Ok(result)
}

fn update_backup_records(
  db: &Database,
  source_root: &str,
  dest_path: &str,
  result: &BackupResult,
) -> Result<(), String> {
  let conn = db.conn.lock().map_err(|e| e.to_string())?;

  // 更新成功备份的记录
  let _ = conn.execute(
    "UPDATE files SET status = 'backed_up', dest_path = ?1, backed_up_at = datetime('now')
     WHERE source_root = ?2 AND status = 'pending'",
    rusqlite::params![dest_path, source_root],
  );

  // 标记失败的文件
  for f in &result.failed_files {
    let _ = conn.execute(
      "UPDATE files SET status = 'failed', error_message = ?1
       WHERE source_root = ?2 AND relative_path = ?3",
      rusqlite::params![f.error, source_root, f.relative_path],
    );
  }

  Ok(())
}

/// 查询指定源目录下的失败文件
#[tauri::command]
pub async fn get_failed_files(
  source_root: String,
  db: State<'_, Database>,
) -> Result<Vec<FailedFile>, String> {
  let conn = db.conn.lock().map_err(|e| e.to_string())?;
  let mut stmt = conn
    .prepare(
      "SELECT relative_path, file_size, error_message
       FROM files WHERE source_root = ?1 AND status = 'failed' AND error_message IS NOT NULL",
    )
    .map_err(|e| e.to_string())?;

  let rows = stmt
    .query_map(rusqlite::params![source_root], |row| {
      Ok(FailedFile {
        source_path: String::new(),
        relative_path: row.get(0)?,
        file_size: row.get::<_, i64>(1)? as u64,
        error: row.get::<_, String>(2)?,
      })
    })
    .map_err(|e| e.to_string())?;

  let mut files = Vec::new();
  for row in rows {
    files.push(row.map_err(|e| e.to_string())?);
  }
  Ok(files)
}

/// 重试备份失败的文件
#[tauri::command]
pub async fn retry_failed_backup(
  app: AppHandle,
  source_root: String,
  dest_path: String,
  files: Vec<FailedFile>,
  conflict_strategy: Option<String>,
  concurrency: Option<usize>,
  db: State<'_, Database>,
  state: State<'_, BackupState>,
) -> Result<BackupResult, String> {
  state.cancel.store(false, Ordering::SeqCst);
  state.paused.store(false, Ordering::SeqCst);

  let dest_root = std::path::Path::new(&dest_path);
  if !dest_root.exists() {
    std::fs::create_dir_all(dest_root).map_err(|e| e.to_string())?;
  }

  let total_files = files.len() as u64;
  let total_bytes: u64 = files.iter().map(|f| f.file_size).sum();
  let concurrency = concurrency.unwrap_or(4).clamp(1, 16);
  let strategy = ConflictStrategy::from_str(
    conflict_strategy.as_deref().unwrap_or("rename"),
  );

  let backup_files: Vec<BackupFile> = files
    .into_iter()
    .map(|f| BackupFile {
      source_path: std::path::Path::new(&source_root)
        .join(&f.relative_path)
        .to_string_lossy()
        .to_string(),
      relative_path: f.relative_path,
      file_size: f.file_size,
    })
    .collect();

  let progress = BackupProgressEmitter::new(Some(Arc::new(app.clone())), total_files, total_bytes);

  progress.emit_log(&format!(
    "Retrying backup: {} files, {} bytes, {} threads",
    total_files, total_bytes, concurrency
  ));

  let source_root = source_root.trim_end_matches(std::path::MAIN_SEPARATOR_STR).to_string();

  let task_id = create_backup_task(&db, &source_root, &dest_path, total_files, total_bytes)
    .map_err(|e| format!("Failed to create backup task: {}", e))?;

  let copier = FileCopier::new(concurrency, strategy);
  let result = copier.run(
    backup_files,
    dest_root,
    progress.clone(),
    state.cancel.clone(),
    state.paused.clone(),
  )?;

  update_backup_task(
    &db,
    task_id,
    result.succeeded,
    result.copied_bytes,
    result.failed,
    "completed",
    result.avg_speed_mbps,
  );

  if let Err(e) = update_backup_records(&db, &source_root, &dest_path, &result) {
    progress.emit_log(&format!("Warning: failed to update database: {}", e));
  }

  progress.emit_log(&format!(
    "Retry complete: {} succeeded, {} failed",
    result.succeeded, result.failed
  ));

  app
    .emit("backup:complete", &result)
    .map_err(|e| e.to_string())?;

  Ok(result)
}

#[tauri::command]
pub async fn pause_backup(state: State<'_, BackupState>) -> Result<(), String> {
  state.paused.store(true, Ordering::SeqCst);
  Ok(())
}

#[tauri::command]
pub async fn resume_backup(state: State<'_, BackupState>) -> Result<(), String> {
  state.paused.store(false, Ordering::SeqCst);
  Ok(())
}

#[tauri::command]
pub async fn cancel_backup(state: State<'_, BackupState>) -> Result<(), String> {
  state.cancel.store(true, Ordering::SeqCst);
  state.paused.store(false, Ordering::SeqCst);
  Ok(())
}

#[tauri::command]
pub async fn get_backup_history(
  db: State<'_, Database>,
) -> Result<Vec<BackupHistoryEntry>, String> {
  let conn = db.conn.lock().map_err(|e| e.to_string())?;
  queries::get_backup_history(&conn)
}
