use std::path::Path;

use crate::hasher::fast_hash::verify_file_integrity;
use crate::hasher::sha256::verify_sha256;
use crate::scanner::FileEntry;

use super::queries;
use super::Database;

/// 抽取比对所需的数据库字段，避免在哈希计算期间长时间持有数据库锁。
struct CompareMeta {
  dest_path: String,
  file_size: i64,
  modified_at: i64,
  backed_up: bool,
}

fn hash_matches(strategy: &str, source: &Path, dest: &Path) -> bool {
  match strategy {
    "fast_hash" => verify_file_integrity(source, dest).unwrap_or(false),
    "sha256" => verify_sha256(source, dest).unwrap_or(false),
    _ => false,
  }
}

/// 将扫描结果与数据库记录比对，返回带状态的文件列表
pub fn compute_file_statuses(
  db: &Database,
  source_root: &str,
  scanned_files: Vec<FileEntry>,
) -> Result<Vec<FileEntry>, String> {
  let (strategy, record_by_rel) = {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let strategy = queries::get_setting(&conn, "compare_strategy")?
      .unwrap_or_else(|| "size_time".to_string());
    let records = queries::get_files_by_source(&conn, source_root)?;

    let map: std::collections::HashMap<String, CompareMeta> = records
      .into_iter()
      .map(|r| {
        (
          r.relative_path,
          CompareMeta {
            dest_path: r.dest_path,
            file_size: r.file_size,
            modified_at: r.modified_at,
            backed_up: matches!(r.status.as_str(), "backed_up" | "verified"),
          },
        )
      })
      .collect();

    (strategy, map)
  };

  let mut result = Vec::with_capacity(scanned_files.len());

  for mut file in scanned_files {
    match record_by_rel.get(file.relative_path.as_str()) {
      Some(record) => {
        let size_match = record.file_size == file.file_size as i64;
        let same = if record.backed_up && size_match {
          if strategy == "fast_hash" || strategy == "sha256" {
            // 哈希策略忽略时间戳，直接比对内容；目标文件缺失视为已变更。
            !record.dest_path.is_empty()
              && hash_matches(&strategy, Path::new(&file.path), Path::new(&record.dest_path))
          } else {
            record.modified_at == file.modified_at
          }
        } else {
          false
        };

        file.status = if same { "backed_up" } else { "changed" }.to_string();
      }
      None => {
        file.status = "new".to_string();
      }
    }
    result.push(file);
  }

  Ok(result)
}
