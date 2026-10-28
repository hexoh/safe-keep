use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::UNIX_EPOCH;

use tauri::{AppHandle, Emitter, State};

use crate::copier::conflict::ConflictStrategy;
use crate::copier::parallel::FileCopier;
use crate::copier::progress::BackupProgressEmitter;
use crate::copier::{BackupFile, BackupResult, FailedFile};
use crate::database::models::{BackupHistoryEntry, BackupTask, FileRecord};
use crate::database::queries;
use crate::database::Database;
use crate::device::list::available_space;

const ERR_DISK_FULL: &str = "ERR_DISK_FULL";

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

fn record_meta(source_path: &str) -> (String, i64) {
  let path = Path::new(source_path);
  let file_name = path
    .file_name()
    .map(|n| n.to_string_lossy().to_string())
    .unwrap_or_default();
  let modified_at = fs::metadata(path)
    .ok()
    .and_then(|m| m.modified().ok())
    .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
    .map(|d| d.as_secs() as i64)
    .unwrap_or(0);
  (file_name, modified_at)
}

/// 备份开始前把本次要复制的文件写入 files 表（状态 pending）。
///
/// 这是清理 / 历史 / 还原页面的数据来源；之前只做对比不落库，导致这些页面永远为空。
fn upsert_pending_records(
  db: &Database,
  source_root: &str,
  dest_root: &Path,
  files: &[BackupFile],
) -> Result<(), String> {
  let mut conn = db.conn.lock().map_err(|e| e.to_string())?;
  let tx = conn.transaction().map_err(|e| e.to_string())?;

  for file in files {
    let (file_name, modified_at) = record_meta(&file.source_path);
    let dest_path = dest_root
      .join(file.dest_relative())
      .to_string_lossy()
      .to_string();

    tx.execute(
      "INSERT INTO files
         (source_root, relative_path, dest_path, file_name, file_size, modified_at, status)
       VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'pending')
       ON CONFLICT(source_root, relative_path) DO UPDATE SET
         dest_path = excluded.dest_path,
         file_name = excluded.file_name,
         file_size = excluded.file_size,
         modified_at = excluded.modified_at,
         status = 'pending',
         error_message = NULL",
      rusqlite::params![
        source_root,
        file.relative_path,
        dest_path,
        file_name,
        file.file_size as i64,
        modified_at
      ],
    )
    .map_err(|e| e.to_string())?;
  }

  tx.commit().map_err(|e| e.to_string())
}

/// 按复制结果精确更新本次涉及的文件，避免误改同源目录下其他文件的状态。
fn update_backup_records(
  db: &Database,
  source_root: &str,
  dest_root: &Path,
  files: &[BackupFile],
  result: &BackupResult,
) -> Result<(), String> {
  let succeeded: HashSet<&str> = result
    .succeeded_files
    .iter()
    .map(String::as_str)
    .collect();
  let failed: HashMap<&str, &str> = result
    .failed_files
    .iter()
    .map(|f| (f.relative_path.as_str(), f.error.as_str()))
    .collect();

  let conn = db.conn.lock().map_err(|e| e.to_string())?;

  for file in files {
    let dest_path = dest_root
      .join(file.dest_relative())
      .to_string_lossy()
      .to_string();

    if let Some(error) = failed.get(file.relative_path.as_str()) {
      let _ = conn.execute(
        "UPDATE files SET status = 'failed', error_message = ?1, dest_path = ?2
         WHERE source_root = ?3 AND relative_path = ?4",
        rusqlite::params![error, dest_path, source_root, file.relative_path],
      );
    } else if succeeded.contains(file.relative_path.as_str()) {
      let _ = conn.execute(
        "UPDATE files SET status = 'backed_up', error_message = NULL, dest_path = ?1,
         backed_up_at = datetime('now')
         WHERE source_root = ?2 AND relative_path = ?3",
        rusqlite::params![dest_path, source_root, file.relative_path],
      );
    }
    // 未处理（例如任务被取消）的文件保持 pending，下次扫描会重新纳入。
  }

  Ok(())
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
#[serde(rename_all = "camelCase")]
pub struct StartBackupOptions {
  pub source_root: String,
  pub dest_path: String,
  pub files: Vec<BackupFile>,
  pub conflict_strategy: Option<String>,
  pub concurrency: Option<usize>,
}

/// 备份前检查目标卷剩余空间，空间不足时返回可被前端识别的错误码。
fn ensure_disk_space(dest_root: &Path, required: u64) -> Result<(), String> {
  if let Some(available) = available_space(dest_root) {
    if required > available {
      return Err(format!(
        "{}: need {} bytes, only {} bytes available",
        ERR_DISK_FULL, required, available
      ));
    }
  }
  Ok(())
}

fn canonical_source_root(source_root: &str) -> String {
  fs::canonicalize(source_root)
    .map(|p| p.to_string_lossy().to_string())
    .unwrap_or_else(|_| {
      source_root
        .trim_end_matches(std::path::MAIN_SEPARATOR_STR)
        .to_string()
    })
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

  ensure_disk_space(dest_root, total_bytes)?;

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

  let source_root = canonical_source_root(&options.source_root);

  // 创建备份任务记录
  let task_id = create_backup_task(&db, &source_root, &options.dest_path, total_files, total_bytes)
    .map_err(|e| format!("Failed to create backup task: {}", e))?;

  // 写入待备份文件记录（files 表是清理 / 历史 / 还原的数据来源）
  upsert_pending_records(&db, &source_root, dest_root, &options.files)
    .map_err(|e| format!("Failed to record files: {}", e))?;

  let copier = FileCopier::new(concurrency, strategy);
  let result = copier.run(
    options.files.clone(),
    dest_root,
    progress.clone(),
    state.cancel.clone(),
    state.paused.clone(),
  )?;

  // 先写文件记录、再落任务完成时间：保证 backed_up_at <= completed_at，
  // 历史详情页按任务时间窗口筛选成功文件时才不会漏数据。
  if let Err(e) = update_backup_records(&db, &source_root, dest_root, &options.files, &result) {
    progress.emit_log(&format!("Warning: failed to update database: {}", e));
  }

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

  progress.emit_log(&format!(
    "Backup complete: {} succeeded, {} failed, {:.2} MB/s avg",
    result.succeeded, result.failed, result.avg_speed_mbps
  ));

  app
    .emit("backup:complete", &result)
    .map_err(|e| e.to_string())?;

  Ok(result)
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
      "SELECT relative_path, file_size, error_message, dest_path
       FROM files WHERE source_root = ?1 AND status = 'failed' AND error_message IS NOT NULL",
    )
    .map_err(|e| e.to_string())?;

  let rows = stmt
    .query_map(rusqlite::params![source_root], |row| {
      let relative_path: String = row.get(0)?;
      Ok(FailedFile {
        source_path: Path::new(&source_root)
          .join(&relative_path)
          .to_string_lossy()
          .to_string(),
        relative_path,
        dest_path: Some(row.get::<_, String>(3)?),
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
  ensure_disk_space(dest_root, total_bytes)?;
  let concurrency = concurrency.unwrap_or(4).clamp(1, 16);
  let strategy = ConflictStrategy::from_str(
    conflict_strategy.as_deref().unwrap_or("rename"),
  );

  let backup_files: Vec<BackupFile> = files
    .into_iter()
    .map(|f| {
      // 失败记录中保存了完整目标路径；按目标根目录还原出相对路径，
      // 这样按日期归档 / 平铺的任务重试时不会把文件写回源目录结构。
      let dest_relative_path = f
        .dest_path
        .as_deref()
        .and_then(|p| Path::new(p).strip_prefix(dest_root).ok())
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| f.relative_path.clone());

      BackupFile {
        source_path: Path::new(&source_root)
          .join(&f.relative_path)
          .to_string_lossy()
          .to_string(),
        relative_path: f.relative_path,
        dest_relative_path: Some(dest_relative_path),
        file_size: f.file_size,
      }
    })
    .collect();

  let progress = BackupProgressEmitter::new(Some(Arc::new(app.clone())), total_files, total_bytes);

  progress.emit_log(&format!(
    "Retrying backup: {} files, {} bytes, {} threads",
    total_files, total_bytes, concurrency
  ));

  let source_root = canonical_source_root(&source_root);

  let task_id = create_backup_task(&db, &source_root, &dest_path, total_files, total_bytes)
    .map_err(|e| format!("Failed to create backup task: {}", e))?;

  upsert_pending_records(&db, &source_root, dest_root, &backup_files)
    .map_err(|e| format!("Failed to record files: {}", e))?;

  let copier = FileCopier::new(concurrency, strategy);
  let result = copier.run(
    backup_files.clone(),
    dest_root,
    progress.clone(),
    state.cancel.clone(),
    state.paused.clone(),
  )?;

  if let Err(e) = update_backup_records(&db, &source_root, dest_root, &backup_files, &result) {
    progress.emit_log(&format!("Warning: failed to update database: {}", e));
  }

  // 同上：文件记录先落库，任务完成时间后写，保证任务时间窗口覆盖到本次文件。
  update_backup_task(
    &db,
    task_id,
    result.succeeded,
    result.copied_bytes,
    result.failed,
    "completed",
    result.avg_speed_mbps,
  );

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

/// 备份任务列表（历史记录页使用，按时间倒序）
#[tauri::command]
pub async fn get_backup_tasks(db: State<'_, Database>) -> Result<Vec<BackupTask>, String> {
  let conn = db.conn.lock().map_err(|e| e.to_string())?;
  queries::get_backup_tasks(&conn)
}

/// 查询指定源目录下的文件记录（历史详情页展示成功 / 失败文件列表）
#[tauri::command]
pub async fn get_source_files(
  source_root: String,
  statuses: Vec<String>,
  from: Option<String>,
  to: Option<String>,
  db: State<'_, Database>,
) -> Result<Vec<FileRecord>, String> {
  let conn = db.conn.lock().map_err(|e| e.to_string())?;
  queries::get_files_by_source_statuses(
    &conn,
    &source_root,
    &statuses,
    from.as_deref(),
    to.as_deref(),
  )
}
