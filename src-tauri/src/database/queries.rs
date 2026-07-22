use rusqlite::{params, Connection};

use super::models::FileRecord;
use crate::deleter::CleanupFile;

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

/// 查询所有已备份且源端存在的文件（用于清理筛选）
pub fn get_backed_up_files(
  conn: &Connection,
  source_root: &str,
) -> Result<Vec<FileRecord>, String> {
  let mut stmt = conn
    .prepare(
      "SELECT id, source_root, relative_path, dest_path, file_name,
              file_size, modified_at, status, backed_up_at, verified_at, error_message
       FROM files
       WHERE source_root = ? AND status IN ('backed_up', 'verified')",
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

/// 筛选可清理文件 — 根据时间、类型、大小过滤已备份文件
pub fn get_cleanup_candidates(
  conn: &Connection,
  source_root: &str,
  before_days: Option<u64>,
  is_image: Option<bool>,
  min_size: Option<i64>,
  max_size: Option<i64>,
) -> Result<Vec<CleanupFile>, String> {
  let mut sql = String::from(
    "SELECT id, source_root, relative_path, dest_path, file_name,
            file_size, modified_at, backed_up_at
     FROM files
     WHERE source_root = ?1 AND status IN ('backed_up', 'verified')",
  );

  if before_days.is_some() {
    sql.push_str(" AND backed_up_at IS NOT NULL AND julianday('now') - julianday(backed_up_at) > ?2");
  }
  if let Some(img) = is_image {
    if img {
      sql.push_str(
        " AND (LOWER(file_name) LIKE '%.jpg' OR LOWER(file_name) LIKE '%.jpeg'
              OR LOWER(file_name) LIKE '%.png' OR LOWER(file_name) LIKE '%.heic'
              OR LOWER(file_name) LIKE '%.webp' OR LOWER(file_name) LIKE '%.gif'
              OR LOWER(file_name) LIKE '%.bmp' OR LOWER(file_name) LIKE '%.tiff'
              OR LOWER(file_name) LIKE '%.raw')",
      );
    } else {
      sql.push_str(
        " AND (LOWER(file_name) LIKE '%.mp4' OR LOWER(file_name) LIKE '%.mov'
              OR LOWER(file_name) LIKE '%.avi' OR LOWER(file_name) LIKE '%.mkv'
              OR LOWER(file_name) LIKE '%.mts' OR LOWER(file_name) LIKE '%.m2ts')",
      );
    }
  }
  if min_size.is_some() {
    sql.push_str(" AND file_size >= ?3");
  }
  if max_size.is_some() {
    sql.push_str(" AND file_size <= ?4");
  }

  let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;

  let records = stmt
    .query_map(
      rusqlite::params![
        source_root,
        before_days.unwrap_or(0) as i64,
        min_size.unwrap_or(0),
        max_size.unwrap_or(i64::MAX),
      ],
      |row| {
        Ok(CleanupFile {
          file_id: row.get(0)?,
          source_root: row.get(1)?,
          relative_path: row.get(2)?,
          dest_path: row.get(3)?,
          file_name: row.get(4)?,
          file_size: row.get(5)?,
          modified_at: row.get(6)?,
          backed_up_at: row.get(7)?,
        })
      },
    )
    .map_err(|e| e.to_string())?;

  let mut files = Vec::new();
  for record in records {
    files.push(record.map_err(|e| e.to_string())?);
  }
  Ok(files)
}

/// 获取所有去重的源根目录列表
pub fn get_source_roots(conn: &Connection) -> Result<Vec<String>, String> {
  let mut stmt = conn
    .prepare("SELECT DISTINCT source_root FROM files ORDER BY source_root")
    .map_err(|e| e.to_string())?;

  let roots = stmt
    .query_map([], |row| row.get::<_, String>(0))
    .map_err(|e| e.to_string())?;

  let mut result = Vec::new();
  for r in roots {
    result.push(r.map_err(|e| e.to_string())?);
  }
  Ok(result)
}
