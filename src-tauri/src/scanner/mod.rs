pub mod filter;
pub mod progress;
pub mod walk;

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct FileEntry {
  pub path: String,
  pub relative_path: String,
  pub file_name: String,
  pub file_size: u64,
  pub modified_at: i64,
  pub is_image: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScanProgress {
  pub scanned: u64,
  pub total: Option<u64>,
  pub current_file: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScanResult {
  pub files: Vec<FileEntry>,
  pub total_files: u64,
  pub total_size: u64,
}
