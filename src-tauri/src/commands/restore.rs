use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tauri::{AppHandle, Emitter, State};

use crate::database::Database;
use crate::restorer::conflict::RestoreConflictStrategy;
use crate::restorer::restore::restore_files;
use crate::restorer::{RestoreFile, RestoreProgress, RestoreResult};

pub struct RestoreState {
  pub cancel: Arc<AtomicBool>,
}

impl RestoreState {
  pub fn new() -> Self {
    Self {
      cancel: Arc::new(AtomicBool::new(false)),
    }
  }
}

#[derive(serde::Deserialize)]
pub struct StartRestoreOptions {
  pub source_root: String,
  pub restore_target: String,
  pub files: Vec<RestoreFile>,
  pub conflict_strategy: Option<String>,
}

#[tauri::command]
pub async fn get_restorable_files(
  db: State<'_, Database>,
) -> Result<Vec<RestoreFile>, String> {
  let conn = db.conn.lock().map_err(|e| e.to_string())?;

  let mut stmt = conn
    .prepare(
      "SELECT id, source_root, relative_path, dest_path, file_name,
              file_size, modified_at, backed_up_at
       FROM files WHERE status = 'deleted' ORDER BY backed_up_at DESC",
    )
    .map_err(|e| e.to_string())?;

  let records = stmt
    .query_map([], |row| {
      Ok(RestoreFile {
        file_id: row.get(0)?,
        source_root: row.get(1)?,
        relative_path: row.get(2)?,
        dest_path: row.get(3)?,
        file_name: row.get(4)?,
        file_size: row.get(5)?,
        modified_at: row.get(6)?,
        backed_up_at: row.get(7)?,
      })
    })
    .map_err(|e| e.to_string())?;

  let mut files = Vec::new();
  for record in records {
    files.push(record.map_err(|e| e.to_string())?);
  }
  Ok(files)
}

#[tauri::command]
pub async fn start_restore(
  app: AppHandle,
  options: StartRestoreOptions,
  db: State<'_, Database>,
  state: State<'_, RestoreState>,
) -> Result<RestoreResult, String> {
  state.cancel.store(false, Ordering::SeqCst);

  let target_root = Path::new(&options.restore_target);
  if !target_root.exists() {
    std::fs::create_dir_all(target_root).map_err(|e| e.to_string())?;
  }

  let strategy = RestoreConflictStrategy::from_str(
    options
      .conflict_strategy
      .as_deref()
      .unwrap_or("overwrite"),
  );

  let file_count = options.files.len();
  let app_handle = Arc::new(app.clone());

  app_handle
    .emit(
      "restore:log",
      serde_json::json!({"message": format!("Starting restore: {} files", file_count)}),
    )
    .ok();

  let result = restore_files(
    options.files,
    target_root,
    strategy,
    state.cancel.clone(),
    |total, processed, succeeded, failed, current_file| {
      let _ = app_handle.emit(
        "restore:progress",
        RestoreProgress {
          total_files: total,
          processed,
          succeeded,
          failed,
          current_file: current_file.to_string(),
        },
      );
    },
  );

  // Update database: mark successfully restored files
  if result.succeeded > 0 {
    if let Ok(conn) = db.conn.lock() {
      let source_root = options.source_root.trim_end_matches(std::path::MAIN_SEPARATOR_STR);
      let _ = conn.execute(
        "UPDATE files SET status = 'restored' WHERE source_root = ?1 AND status = 'deleted'",
        rusqlite::params![source_root],
      );
    }
  }

  app_handle
    .emit("restore:complete", &result)
    .map_err(|e| e.to_string())?;

  Ok(result)
}

#[tauri::command]
pub async fn cancel_restore(state: State<'_, RestoreState>) -> Result<(), String> {
  state.cancel.store(true, Ordering::SeqCst);
  Ok(())
}
