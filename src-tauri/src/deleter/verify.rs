use std::path::Path;

use crate::hasher::fast_hash::verify_file_integrity;

/// 双重校验：文件存在 + 大小/时间一致 + 头尾哈希
pub fn verify_backup_integrity(source: &Path, dest: &Path) -> Result<bool, String> {
  if !source.exists() {
    return Err(format!("Source file does not exist: {}", source.display()));
  }
  if !dest.exists() {
    return Err(format!("Dest file does not exist: {}", dest.display()));
  }

  let src_meta = source.metadata().map_err(|e| e.to_string())?;
  let dst_meta = dest.metadata().map_err(|e| e.to_string())?;

  if src_meta.len() != dst_meta.len() {
    return Ok(false);
  }

  let src_mtime = src_meta
    .modified()
    .ok()
    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
    .map(|d| d.as_secs() as i64)
    .unwrap_or(0);
  let dst_mtime = dst_meta
    .modified()
    .ok()
    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
    .map(|d| d.as_secs() as i64)
    .unwrap_or(0);

  if src_mtime != dst_mtime {
    return Ok(false);
  }

  verify_file_integrity(source, dest)
}

/// 仅校验大小和修改时间（快速路径，用于批量扫描）
pub fn quick_verify(source: &Path, dest: &Path) -> Result<bool, String> {
  if !source.exists() || !dest.exists() {
    return Ok(false);
  }

  let src_meta = source.metadata().map_err(|e| e.to_string())?;
  let dst_meta = dest.metadata().map_err(|e| e.to_string())?;

  Ok(src_meta.len() == dst_meta.len() && src_meta.modified().ok() == dst_meta.modified().ok())
}
