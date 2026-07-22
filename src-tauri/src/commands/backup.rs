use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tauri::{AppHandle, Emitter, State};

use crate::copier::conflict::ConflictStrategy;
use crate::copier::parallel::FileCopier;
use crate::copier::progress::BackupProgressEmitter;
use crate::copier::{BackupFile, BackupResult};
use crate::database::Database;

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

  let copier = FileCopier::new(concurrency, strategy);
  let result = copier.run(
    options.files,
    dest_root,
    progress.clone(),
    state.cancel.clone(),
    state.paused.clone(),
  )?;

  // 更新数据库
  let source_root = options
    .source_root
    .trim_end_matches(std::path::MAIN_SEPARATOR_STR)
    .to_string();

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

#[allow(unused_variables)]
fn update_backup_records(
  db: &Database,
  source_root: &str,
  dest_path: &str,
  result: &BackupResult,
) -> Result<(), String> {
  let conn = db.conn.lock().map_err(|e| e.to_string())?;

  // 更新扫描时已存在的记录
  let _ = conn.execute(
    "UPDATE files SET status = 'backed_up', dest_path = ?1, backed_up_at = datetime('now')
     WHERE source_root = ?2 AND status = 'pending'",
    rusqlite::params![dest_path, source_root],
  );

  Ok(())
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
