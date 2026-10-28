pub mod conflict;
pub mod restore;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreFile {
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
pub struct RestoreProgress {
  pub total_files: u64,
  pub processed: u64,
  pub succeeded: u64,
  pub failed: u64,
  pub current_file: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RestoreResult {
  pub total_files: u64,
  pub succeeded: u64,
  pub failed: u64,
  pub total_bytes: u64,
  pub restored_bytes: u64,
  pub duration_secs: f64,
  pub errors: Vec<String>,
  /// 是否因目标设备掉线而提前中止。
  #[serde(default)]
  pub aborted: bool,
  /// 成功还原的文件 ID，仅用于后端更新数据库，不发送给前端。
  #[serde(skip)]
  pub succeeded_file_ids: Vec<i64>,
}
