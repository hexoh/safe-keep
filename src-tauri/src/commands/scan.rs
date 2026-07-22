use std::path::Path;
use std::sync::Arc;

use tauri::{AppHandle, Emitter, State};

use crate::scanner::progress::ScanProgressEmitter;
use crate::scanner::walk::FileScanner;
use crate::scanner::ScanResult;

#[derive(serde::Deserialize)]
pub struct ScanOptions {
  pub source_path: String,
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
) -> Result<ScanResult, String> {
  let path = Path::new(&options.source_path);
  if !path.exists() {
    return Err(format!("Path does not exist: {}", options.source_path));
  }
  if !path.is_dir() {
    return Err(format!("Path is not a directory: {}", options.source_path));
  }
  let scanner = FileScanner::new();
  {
    let mut guard = state.scanner.lock().map_err(|e| e.to_string())?;
    *guard = Some(scanner);
  }

  let progress = ScanProgressEmitter::new(Some(Arc::new(app.clone())));

  let mut result = {
    let guard = state.scanner.lock().map_err(|e| e.to_string())?;
    let scanner = guard.as_ref().ok_or("No scanner initialized")?;
    scanner.scan(path, progress)?
  };

  if let Some(min) = options.min_size {
    result.files.retain(|f| f.file_size >= min);
  }
  if let Some(max) = options.max_size {
    result.files.retain(|f| f.file_size <= max);
  }
  if options.include_images == Some(false) {
    result.files.retain(|f| !f.is_image);
  }
  if options.include_videos == Some(false) {
    result.files.retain(|f| f.is_image);
  }

  result.total_files = result.files.len() as u64;
  result.total_size = result.files.iter().map(|f| f.file_size).sum();

  app.emit("scan:complete", serde_json::json!({ "total": result.total_files }))
    .map_err(|e| e.to_string())?;

  Ok(result)
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
