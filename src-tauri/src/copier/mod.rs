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
  /// 扫描阶段已取得的修改时间（Unix 秒）。
  ///
  /// 带上它可以省掉备份前为每个文件重新 stat 源端的开销：在 MTP 设备上，
  /// 那是一次完整的目录树串行往返（每个文件一次 GetObjectInfo）。
  #[serde(default)]
  pub modified_at: i64,
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
  /// 数据库里记录的修改时间，重试时透传，避免再逐个 stat 源端。
  #[serde(default)]
  pub modified_at: i64,
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
  /// 是否因设备级故障（掉线 / 挂载点失效）提前中止。
  #[serde(default)]
  pub aborted: bool,
  /// 成功复制的相对路径，仅用于后端更新数据库。
  #[serde(skip)]
  pub succeeded_files: Vec<String>,
}
