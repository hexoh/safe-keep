use rusqlite::{params, Connection};

use super::models::FileRecord;

pub fn get_files_by_source(
  conn: &Connection,
  source_root: &str,
) -> Result<Vec<FileRecord>, String> {
  let mut stmt = conn
    .prepare(
      "SELECT id, source_root, relative_path, dest_path, file_name,
              file_size, modified_at, status, backed_up_at, verified_at, error_message
       FROM files WHERE source_root = ?",
    )
    .map_err(|e| e.to_string())?;

  let records = stmt
    .query_map(params![source_root], |row| {
      Ok(FileRecord {
        id: row.get(0)?,
        source_root: row.get(1)?,
        relative_path: row.get(2)?,
        dest_path: row.get(3)?,
        file_name: row.get(4)?,
        file_size: row.get(5)?,
        modified_at: row.get(6)?,
        status: row.get(7)?,
        backed_up_at: row.get(8)?,
        verified_at: row.get(9)?,
        error_message: row.get(10)?,
      })
    })
    .map_err(|e| e.to_string())?;

  let mut files = Vec::new();
  for record in records {
    files.push(record.map_err(|e| e.to_string())?);
  }
  Ok(files)
}
