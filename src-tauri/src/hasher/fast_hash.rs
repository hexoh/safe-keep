use std::io::Read;
use std::path::Path;

use xxhash_rust::xxh3::Xxh3;

const HEAD_SIZE: usize = 4096;
const TAIL_SIZE: usize = 4096;

/// 计算文件头尾 xxHash 用于快速校验
pub fn compute_head_tail_hash(path: &Path) -> Result<(u64, u64), String> {
  let mut file = std::fs::File::open(path).map_err(|e| e.to_string())?;
  let file_len = file.metadata().map_err(|e| e.to_string())?.len();

  // 头 4KB hash
  let head_hash = {
    let mut hasher = Xxh3::new();
    let mut buf = vec![0u8; HEAD_SIZE.min(file_len as usize)];
    let n = file.read(&mut buf).map_err(|e| e.to_string())?;
    hasher.update(&buf[..n]);
    hasher.digest()
  };

  // 尾 4KB hash
  let tail_hash = if file_len > HEAD_SIZE as u64 {
    let tail_start = file_len.saturating_sub(TAIL_SIZE as u64);
    let tail_len = (file_len - tail_start) as usize;

    let mut file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    use std::io::Seek;
    file.seek(std::io::SeekFrom::Start(tail_start))
      .map_err(|e| e.to_string())?;

    let mut hasher = Xxh3::new();
    let mut buf = vec![0u8; tail_len];
    let n = file.read(&mut buf).map_err(|e| e.to_string())?;
    hasher.update(&buf[..n]);
    hasher.digest()
  } else {
    head_hash
  };

  Ok((head_hash, tail_hash))
}

/// 比较源和目标文件的头尾 hash 是否一致
pub fn verify_file_integrity(source: &Path, dest: &Path) -> Result<bool, String> {
  let (src_head, src_tail) = compute_head_tail_hash(source)?;
  let (dst_head, dst_tail) = compute_head_tail_hash(dest)?;
  Ok(src_head == dst_head && src_tail == dst_tail)
}
