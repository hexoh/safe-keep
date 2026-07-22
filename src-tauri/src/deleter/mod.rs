pub mod delete;
pub mod verify;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupFile {
  pub file_id: i64,
  pub source_root: String,
  pub relative_path: String,
  pub dest_path: String,
  pub file_name: String,
  pub file_size: i64,
  pub modified_at: i64,
  pub backed_up_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DryRunResult {
  pub files: Vec<CleanupFile>,
  pub total_files: u64,
  pub total_size: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CleanupProgress {
  pub total_files: u64,
  pub processed: u64,
  pub succeeded: u64,
  pub failed: u64,
  pub current_file: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CleanupResult {
  pub total_files: u64,
  pub succeeded: u64,
  pub failed: u64,
  pub total_size: u64,
  pub freed_size: u64,
  pub errors: Vec<String>,
}
