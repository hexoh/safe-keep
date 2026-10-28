use std::path::Path;

use rayon::prelude::*;

use crate::device::profile::DeviceProfile;
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

fn status_for(record: Option<&CompareMeta>, file: &FileEntry, strategy: &str) -> &'static str {
  match record {
    Some(record) => {
      let size_match = record.file_size == file.file_size as i64;
      let same = if record.backed_up && size_match {
        if strategy == "size_time" {
          // 纯记录比对，不产生任何 I/O —— 慢速设备上的默认策略。
          record.modified_at == file.modified_at
        } else {
          // 哈希策略忽略时间戳，直接比对内容；目标文件缺失视为已变更。
          !record.dest_path.is_empty()
            && hash_matches(strategy, Path::new(&file.path), Path::new(&record.dest_path))
        }
      } else {
        false
      };

      if same {
        "backed_up"
      } else {
        "changed"
      }
    }
    None => "new",
  }
}

/// 将扫描结果与数据库记录比对，返回带状态的文件列表
///
/// 设备策略会覆盖用户设置：MTP / FUSE 设备上不允许逐文件内容哈希，
/// 因为那会退化成"为校验 4KB 而传输整个文件"，把一次扫描变成一次全量下载。
pub fn compute_file_statuses(
  db: &Database,
  source_root: &str,
  profile: DeviceProfile,
  scanned_files: Vec<FileEntry>,
) -> Result<Vec<FileEntry>, String> {
  let (strategy, record_by_rel) = {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let requested = queries::get_setting(&conn, "compare_strategy")?
      .unwrap_or_else(|| "size_time".to_string());
    let strategy = profile.resolve_compare_strategy(&requested);
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
            // restored 表示文件已从备份还原回源端，备份副本仍然有效。
            backed_up: matches!(r.status.as_str(), "backed_up" | "verified" | "restored"),
          },
        )
      })
      .collect();

    (strategy, map)
  };

  let mut files = scanned_files;

  if strategy == "size_time" {
    // 纯内存比对，串行即可，也没有并行的意义。
    for file in files.iter_mut() {
      file.status =
        status_for(record_by_rel.get(file.relative_path.as_str()), file, &strategy).to_string();
    }
    return Ok(files);
  }

  // 哈希比对是 I/O 密集的，按设备能承受的并发上限开小池子，
  // 避免把 U 盘 / 读卡器打爆（全局 rayon 池会一次性上满所有核）。
  let threads = profile.max_concurrency.max(2);
  let pool = rayon::ThreadPoolBuilder::new()
    .num_threads(threads)
    .build()
    .map_err(|e| e.to_string())?;

  pool.install(|| {
    files.par_iter_mut().for_each(|file| {
      file.status =
        status_for(record_by_rel.get(file.relative_path.as_str()), file, &strategy).to_string();
    });
  });

  Ok(files)
}
