use std::path::Path;

use crate::device::profile::DeviceProfile;
use crate::hasher::fast_hash::verify_file_integrity;

/// 双重校验：文件存在 + 大小/时间一致 + 头尾哈希
///
/// 内容哈希只在允许的设备上执行。MTP / FUSE 设备的随机读会被退化成
/// "从头丢弃式读取"，逐文件校验等于把整个文件重新传输一遍——删 1000 张照片
/// 就是再传一遍 1000 张。这类设备改用大小 + 修改时间判定。
pub fn verify_backup_integrity(
  source: &Path,
  dest: &Path,
  profile: &DeviceProfile,
) -> Result<bool, String> {
  // 直接取 metadata，省掉 exists() 带来的额外一次 stat（慢速设备上每次都算往返）。
  let src_meta = source
    .metadata()
    .map_err(|_| format!("Source file does not exist: {}", source.display()))?;
  let dst_meta = dest
    .metadata()
    .map_err(|_| format!("Dest file does not exist: {}", dest.display()))?;

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

  // FAT32 / exFAT 的时间戳精度通常为 2 秒，允许少量误差。
  if (src_mtime - dst_mtime).abs() > 2 {
    return Ok(false);
  }

  if !profile.allows_content_hash {
    return Ok(true);
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
