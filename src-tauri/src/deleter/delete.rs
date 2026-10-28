use std::path::Path;

use crate::device::profile::DeviceProfile;

use super::verify::verify_backup_integrity;

/// 实际使用的删除方式，写进 delete_history 用
pub const METHOD_PERMANENT: &str = "permanent";
pub const METHOD_RECYCLE: &str = "recycle";

/// 执行删除 — 先校验完整性，再执行删除
///
/// 返回实际采用的删除方式：不支持回收站的设备（MTP / FUSE）会强制走永久删除，
/// 否则 `trash` crate 在其上必然失败，调用方记录的历史也会与实际不符。
pub fn delete_file(
  source: &Path,
  dest: &Path,
  permanent: bool,
  profile: &DeviceProfile,
) -> Result<&'static str, String> {
  // 删除前校验
  if !verify_backup_integrity(source, dest, profile)? {
    return Err(format!(
      "Integrity check failed for {}",
      source.display()
    ));
  }

  if permanent || !profile.allows_trash {
    std::fs::remove_file(source).map_err(|e| e.to_string())?;
    Ok(METHOD_PERMANENT)
  } else {
    trash::delete(source).map_err(|e| format!("Failed to move to trash: {}", e))?;
    Ok(METHOD_RECYCLE)
  }
}
