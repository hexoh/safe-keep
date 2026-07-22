use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct BackupHistoryEntry {
  pub source_root: String,
  pub dest_path: String,
  pub total_files: i64,
  pub total_bytes: i64,
  pub backed_up_count: u64,
  pub last_backed_up_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FileRecord {
  pub id: i64,
  pub source_root: String,
  pub relative_path: String,
  pub dest_path: String,
  pub file_name: String,
  pub file_size: i64,
  pub modified_at: i64,
  pub status: String,
  pub backed_up_at: Option<String>,
  pub verified_at: Option<String>,
  pub error_message: Option<String>,
}
