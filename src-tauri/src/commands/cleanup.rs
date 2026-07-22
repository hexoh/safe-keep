use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tauri::{AppHandle, Emitter, State};

use crate::database::queries;
use crate::database::Database;
use crate::deleter::delete::delete_file;
use crate::deleter::{CleanupFile, CleanupProgress, CleanupResult, DryRunResult};

pub struct CleanupState {
  pub cancel: Arc<AtomicBool>,
}

impl CleanupState {
  pub fn new() -> Self {
    Self {
      cancel: Arc::new(AtomicBool::new(false)),
    }
  }
}

#[derive(serde::Deserialize)]
pub struct CleanupFilter {
  pub source_root: String,
  pub before_days: Option<u64>,
  pub is_image: Option<bool>,
  pub min_size: Option<i64>,
  pub max_size: Option<i64>,
}

/// Dry Run — 只列出可清理文件，不执行删除
#[tauri::command]
pub async fn dry_run_cleanup(
  filter: CleanupFilter,
  db: State<'_, Database>,
) -> Result<DryRunResult, String> {
  let conn = db.conn.lock().map_err(|e| e.to_string())?;
  let files = queries::get_cleanup_candidates(
    &conn,
    &filter.source_root,
    filter.before_days,
    filter.is_image,
    filter.min_size,
    filter.max_size,
  )?;

  let total_files = files.len() as u64;
  let total_size: u64 = files.iter().map(|f| f.file_size as u64).sum();

  Ok(DryRunResult {
    files,
    total_files,
    total_size,
  })
}

/// 执行清理 — 删除 Dry Run 中列出的文件
#[tauri::command]
pub async fn execute_cleanup(
  app: AppHandle,
  files: Vec<CleanupFile>,
  permanent: bool,
  db: State<'_, Database>,
  state: State<'_, CleanupState>,
) -> Result<CleanupResult, String> {
  state.cancel.store(false, Ordering::SeqCst);

  let total_files = files.len() as u64;
  let total_size: u64 = files.iter().map(|f| f.file_size as u64).sum();
  let mut succeeded = 0u64;
  let mut failed = 0u64;
  let mut freed = 0u64;
  let mut errors = Vec::new();

  for file in &files {
    if state.cancel.load(Ordering::SeqCst) {
      break;
    }

    let source_path = Path::new(&file.source_root).join(&file.relative_path);
    let dest_path = Path::new(&file.dest_path);

    app
      .emit(
        "cleanup:progress",
        CleanupProgress {
          total_files,
          processed: succeeded + failed,
          succeeded,
          failed,
          current_file: file.file_name.clone(),
        },
      )
      .ok();

    match delete_file(&source_path, dest_path, permanent) {
      Ok(()) => {
        succeeded += 1;
        freed += file.file_size as u64;
        // 更新数据库
        if let Ok(conn) = db.conn.lock() {
          let _ = conn.execute(
            "UPDATE files SET status = 'deleted' WHERE id = ?1",
            rusqlite::params![file.file_id],
          );
          let _ = conn.execute(
            "INSERT INTO delete_history (file_id, source_root, source_path, deleted_at, delete_method)
             VALUES (?1, ?2, ?3, datetime('now'), ?4)",
            rusqlite::params![
              file.file_id,
              file.source_root,
              source_path.to_string_lossy().to_string(),
              if permanent { "permanent" } else { "recycle" },
            ],
          );
        }
      }
      Err(e) => {
        failed += 1;
        errors.push(format!("{}: {}", file.file_name, e));
      }
    }
  }

  let result = CleanupResult {
    total_files,
    succeeded,
    failed,
    total_size,
    freed_size: freed,
    errors,
  };

  app
    .emit("cleanup:complete", &result)
    .map_err(|e| e.to_string())?;

  Ok(result)
}

#[tauri::command]
pub async fn cancel_cleanup(state: State<'_, CleanupState>) -> Result<(), String> {
  state.cancel.store(true, Ordering::SeqCst);
  Ok(())
}

/// 获取有备份记录的源目录列表
#[tauri::command]
pub async fn get_source_roots(db: State<'_, Database>) -> Result<Vec<String>, String> {
  let conn = db.conn.lock().map_err(|e| e.to_string())?;
  queries::get_source_roots(&conn)
}
