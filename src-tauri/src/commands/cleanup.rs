use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tauri::{AppHandle, Emitter, State};

use crate::database::queries;
use crate::database::Database;
use crate::deleter::delete::delete_file;
use crate::deleter::{CleanupFile, CleanupProgress, CleanupResult, DryRunResult};
use crate::device::profile::DeviceProfile;

/// 每批落库的文件数。删除本身在慢速设备上可能要跑很久，
/// 攒批提交既能避免每个文件一次事务（5000 个文件 = 5000 次 WAL 落盘），
/// 又能在中途崩溃时最多只丢一批的状态记录。
const DB_FLUSH_BATCH: usize = 200;

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

impl Default for CleanupState {
  fn default() -> Self {
    Self::new()
  }
}

#[derive(serde::Deserialize)]
pub struct CleanupFilter {
  pub source_root: String,
  pub before_days: Option<u64>,
  pub before_date: Option<String>,
  pub is_image: Option<bool>,
  pub min_size: Option<i64>,
  pub max_size: Option<i64>,
}

struct PendingDeletion {
  file_id: i64,
  source_root: String,
  source_path: String,
  method: &'static str,
}

/// 批量提交删除记录，一个事务完成一批写入。
fn flush_deletions(db: &Database, pending: &mut Vec<PendingDeletion>) -> Result<(), String> {
  if pending.is_empty() {
    return Ok(());
  }

  let mut conn = db.conn.lock().map_err(|e| e.to_string())?;
  let tx = conn.transaction().map_err(|e| e.to_string())?;

  for item in pending.iter() {
    let _ = tx.execute(
      "UPDATE files SET status = 'deleted' WHERE id = ?1",
      rusqlite::params![item.file_id],
    );
    let _ = tx.execute(
      "INSERT INTO delete_history (file_id, source_root, source_path, deleted_at, delete_method)
       VALUES (?1, ?2, ?3, datetime('now'), ?4)",
      rusqlite::params![item.file_id, item.source_root, item.source_path, item.method],
    );
  }

  tx.commit().map_err(|e| e.to_string())?;
  pending.clear();
  Ok(())
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
    filter.before_date.as_deref(),
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
  let mut pending: Vec<PendingDeletion> = Vec::new();
  // 同一源目录只需探测一次设备类型，避免每个文件都 statfs 一遍。
  let mut profiles: HashMap<String, DeviceProfile> = HashMap::new();

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

    let profile = *profiles
      .entry(file.source_root.clone())
      .or_insert_with(|| DeviceProfile::detect(Path::new(&file.source_root)));

    match delete_file(&source_path, dest_path, permanent, &profile) {
      Ok(method) => {
        succeeded += 1;
        freed += file.file_size as u64;
        pending.push(PendingDeletion {
          file_id: file.file_id,
          source_root: file.source_root.clone(),
          source_path: source_path.to_string_lossy().to_string(),
          method,
        });

        if pending.len() >= DB_FLUSH_BATCH {
          flush_deletions(db.inner(), &mut pending)?;
        }
      }
      Err(e) => {
        failed += 1;
        errors.push(format!("{}: {}", file.file_name, e));
      }
    }
  }

  flush_deletions(db.inner(), &mut pending)?;

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
