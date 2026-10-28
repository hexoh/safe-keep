pub mod conflict;
pub mod parallel;
pub mod progress;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupFile {
  pub source_path: String,
  /// 源目录内的相对路径，用于数据库唯一标识与还原。
  pub relative_path: String,
  /// 目标目录内的相对路径（按日期归档 / 平铺时与 relative_path 不同）。
  #[serde(default)]
  pub dest_relative_path: Option<String>,
  pub file_size: u64,
}

impl BackupFile {
  pub fn dest_relative(&self) -> &str {
    self
      .dest_relative_path
      .as_deref()
      .unwrap_or(&self.relative_path)
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedFile {
  pub source_path: String,
  pub relative_path: String,
  #[serde(default)]
  pub dest_path: Option<String>,
  pub file_size: u64,
  pub error: String,
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
  pub failed_files: Vec<FailedFile>,
  /// 成功复制的相对路径，仅用于后端更新数据库。
  #[serde(skip)]
  pub succeeded_files: Vec<String>,
}
