use crate::scanner::FileEntry;

use super::queries;
use super::Database;

/// 将扫描结果与数据库记录比对，返回带状态的文件列表
pub fn compute_file_statuses(
  db: &Database,
  source_root: &str,
  scanned_files: Vec<FileEntry>,
) -> Result<Vec<FileEntry>, String> {
  let conn = db.conn.lock().map_err(|e| e.to_string())?;
  let records = queries::get_files_by_source(&conn, source_root)?;

  let record_by_rel: std::collections::HashMap<&str, &super::models::FileRecord> =
    records.iter().map(|r| (r.relative_path.as_str(), r)).collect();

  let mut result = Vec::with_capacity(scanned_files.len());

  for mut file in scanned_files {
    if let Some(record) = record_by_rel.get(file.relative_path.as_str()) {
      let size_match = record.file_size == file.file_size as i64;
      let time_match = record.modified_at == file.modified_at;

      let backed_up = matches!(record.status.as_str(), "backed_up" | "verified");

      if backed_up && size_match && time_match {
        file.status = "backed_up".to_string();
      } else {
        file.status = "changed".to_string();
      }
    } else {
      file.status = "new".to_string();
    }
    result.push(file);
  }

  Ok(result)
}
