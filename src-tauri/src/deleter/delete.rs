use std::path::Path;

use super::verify::verify_backup_integrity;

/// 执行删除 — 先校验完整性，再执行删除
pub fn delete_file(
  source: &Path,
  dest: &Path,
  permanent: bool,
) -> Result<(), String> {
  // 删除前校验
  if !verify_backup_integrity(source, dest)? {
    return Err(format!(
      "Integrity check failed for {}",
      source.display()
    ));
  }

  if permanent {
    std::fs::remove_file(source).map_err(|e| e.to_string())?;
  } else {
    trash::delete(source).map_err(|e| format!("Failed to move to trash: {}", e))?;
  }

  Ok(())
}
