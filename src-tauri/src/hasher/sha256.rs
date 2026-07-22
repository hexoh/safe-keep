use std::io::Read;
use std::path::Path;

use sha2::{Digest, Sha256};

const BUF_SIZE: usize = 64 * 1024;

/// 计算文件的完整 SHA256 哈希
pub fn compute_sha256(path: &Path) -> Result<String, String> {
  let mut file = std::fs::File::open(path).map_err(|e| e.to_string())?;
  let mut hasher = Sha256::new();
  let mut buf = vec![0u8; BUF_SIZE];

  loop {
    let n = file.read(&mut buf).map_err(|e| e.to_string())?;
    if n == 0 {
      break;
    }
    hasher.update(&buf[..n]);
  }

  let result = hasher.finalize();
  Ok(format!("{:x}", result))
}

/// 比较源和目标文件的 SHA256 是否一致
pub fn verify_sha256(source: &Path, dest: &Path) -> Result<bool, String> {
  let src_hash = compute_sha256(source)?;
  let dst_hash = compute_sha256(dest)?;
  Ok(src_hash == dst_hash)
}
