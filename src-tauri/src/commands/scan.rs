use std::path::Path;
use std::sync::Arc;

use tauri::{AppHandle, Emitter, State};

use crate::database::compare;
use crate::database::Database;
use crate::scanner::progress::ScanProgressEmitter;
use crate::scanner::walk::FileScanner;
use crate::scanner::{FileEntry, ScanResult};

#[derive(serde::Deserialize)]
pub struct ScanOptions {
  pub source_path: String,
  #[allow(dead_code)]
  pub dest_path: Option<String>,
  pub include_images: Option<bool>,
  pub include_videos: Option<bool>,
  pub min_size: Option<u64>,
  pub max_size: Option<u64>,
}

pub struct ScannerState {
  pub scanner: std::sync::Mutex<Option<FileScanner>>,
}

#[tauri::command]
pub async fn start_scan(
  app: AppHandle,
  options: ScanOptions,
  state: State<'_, ScannerState>,
  db: State<'_, Database>,
) -> Result<ScanResult, String> {
  let path = Path::new(&options.source_path);
  if !path.exists() {
    return Err(format!("Path does not exist: {}", options.source_path));
  }
  if !path.is_dir() {
    return Err(format!("Path is not a directory: {}", options.source_path));
  }

  let source_root = path
    .canonicalize()
    .map_err(|e| e.to_string())?
    .to_string_lossy()
    .to_string();

  let scanner = FileScanner::new();
  {
    let mut guard = state.scanner.lock().map_err(|e| e.to_string())?;
    *guard = Some(scanner);
  }

  let progress = ScanProgressEmitter::new(Some(Arc::new(app.clone())));

  let scanned = {
    let guard = state.scanner.lock().map_err(|e| e.to_string())?;
    let scanner = guard.as_ref().ok_or("No scanner initialized")?;
    scanner.scan(path, progress)?
  };

  let mut files: Vec<FileEntry> = scanned.files;

  if let Some(min) = options.min_size {
    files.retain(|f| f.file_size >= min);
  }
  if let Some(max) = options.max_size {
    files.retain(|f| f.file_size <= max);
  }
  if options.include_images == Some(false) {
    files.retain(|f| !f.is_image);
  }
  if options.include_videos == Some(false) {
    files.retain(|f| f.is_image);
  }

  // 增量对比：与数据库记录比对，标记状态
  let files = compare::compute_file_statuses(&db, &source_root, files)?;

  let total_files = files.len() as u64;
  let total_size = files.iter().map(|f| f.file_size).sum();

  let new_count = files.iter().filter(|f| f.status == "new").count();
  let backed_up_count = files.iter().filter(|f| f.status == "backed_up").count();
  let changed_count = files.iter().filter(|f| f.status == "changed").count();

  app
    .emit(
      "scan:complete",
      serde_json::json!({
        "total": total_files,
        "new": new_count,
        "backed_up": backed_up_count,
        "changed": changed_count
      }),
    )
    .map_err(|e| e.to_string())?;

  Ok(ScanResult {
    total_files,
    total_size,
    files,
  })
}

#[tauri::command]
pub async fn cancel_scan(state: State<'_, ScannerState>) -> Result<(), String> {
  let mut guard = state.scanner.lock().map_err(|e| e.to_string())?;
  if let Some(scanner) = guard.as_ref() {
    scanner.cancel();
  }
  *guard = None;
  Ok(())
}
