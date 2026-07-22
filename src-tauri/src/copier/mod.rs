pub mod conflict;
pub mod parallel;
pub mod progress;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupFile {
  pub source_path: String,
  pub relative_path: String,
  pub file_size: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct BackupResult {
  pub total_files: u64,
  pub succeeded: u64,
  pub failed: u64,
  pub total_bytes: u64,
  pub copied_bytes: u64,
  pub duration_secs: f64,
  pub avg_speed_mbps: f64,
  pub errors: Vec<String>,
}
