use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use xxhash_rust::xxh3::Xxh3;

const SNIPPET_SIZE: usize = 4096;

fn hash_bytes(bytes: &[u8]) -> u64 {
  let mut hasher = Xxh3::new();
  hasher.update(bytes);
  hasher.digest()
}

/// 计算文件头尾 xxHash 用于快速校验
///
/// 只打开文件一次：MTP / FUSE 设备上每次 `File::open` 都是一次完整的目标对象打开，
/// 分两次打开会让开销翻倍。
pub fn compute_head_tail_hash(path: &Path) -> Result<(u64, u64), String> {
  let mut file = File::open(path).map_err(|e| e.to_string())?;
  let file_len = file.metadata().map_err(|e| e.to_string())?.len();

  // 头 4KB hash
  let mut head_buf = vec![0u8; SNIPPET_SIZE.min(file_len as usize)];
  let n = file.read(&mut head_buf).map_err(|e| e.to_string())?;
  let head_hash = hash_bytes(&head_buf[..n]);

  // 尾 4KB hash
  let tail_hash = if file_len > SNIPPET_SIZE as u64 {
    let tail_start = file_len - SNIPPET_SIZE as u64;
    let mut tail_buf = vec![0u8; (file_len - tail_start) as usize];
    file
      .seek(SeekFrom::Start(tail_start))
      .map_err(|e| e.to_string())?;
    let n = file.read(&mut tail_buf).map_err(|e| e.to_string())?;
    hash_bytes(&tail_buf[..n])
  } else {
    head_hash
  };

  Ok((head_hash, tail_hash))
}

/// 边读边算的头尾指纹。
///
/// 复制过程中顺流喂数据，就能得到与 `compute_head_tail_hash` 完全一致的
/// (头 hash, 尾 hash)，从而避免复制完再回读一遍源文件——在 MTP 上那等于
/// 把整个文件重新传输一次。
pub struct StreamFingerprint {
  head: Xxh3,
  head_len: usize,
  /// 尾部滚动窗口，只保留最后 SNIPPET_SIZE 字节
  tail: Vec<u8>,
  total: u64,
}

impl StreamFingerprint {
  pub fn new() -> Self {
    Self {
      head: Xxh3::new(),
      head_len: 0,
      tail: Vec::with_capacity(SNIPPET_SIZE),
      total: 0,
    }
  }

  pub fn update(&mut self, buf: &[u8]) {
    if buf.is_empty() {
      return;
    }

    if self.head_len < SNIPPET_SIZE {
      let take = (SNIPPET_SIZE - self.head_len).min(buf.len());
      self.head.update(&buf[..take]);
      self.head_len += take;
    }

    if buf.len() >= SNIPPET_SIZE {
      self.tail.clear();
      self.tail.extend_from_slice(&buf[buf.len() - SNIPPET_SIZE..]);
    } else {
      let overflow = (self.tail.len() + buf.len()).saturating_sub(SNIPPET_SIZE);
      if overflow > 0 {
        self.tail.drain(..overflow);
      }
      self.tail.extend_from_slice(buf);
    }

    self.total += buf.len() as u64;
  }

  /// 返回 (头 hash, 尾 hash)，语义与 `compute_head_tail_hash` 一致：
  /// 文件不超过 4KB 时尾 hash 等于头 hash。
  pub fn finish(self) -> (u64, u64) {
    let head_hash = self.head.digest();
    let tail_hash = if self.total > SNIPPET_SIZE as u64 {
      hash_bytes(&self.tail)
    } else {
      head_hash
    };
    (head_hash, tail_hash)
  }
}

impl Default for StreamFingerprint {
  fn default() -> Self {
    Self::new()
  }
}

/// 比较源和目标文件的头尾 hash 是否一致
pub fn verify_file_integrity(source: &Path, dest: &Path) -> Result<bool, String> {
  let (src_head, src_tail) = compute_head_tail_hash(source)?;
  let (dst_head, dst_tail) = compute_head_tail_hash(dest)?;
  Ok(src_head == dst_head && src_tail == dst_tail)
}

/// 用复制过程中得到的源端指纹校验目标文件，只需读目标端（通常是本地盘）。
pub fn verify_stream_against_dest(fingerprint: (u64, u64), dest: &Path) -> Result<bool, String> {
  let (dst_head, dst_tail) = compute_head_tail_hash(dest)?;
  Ok(fingerprint.0 == dst_head && fingerprint.1 == dst_tail)
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::io::Write;

  fn temp_file(name: &str, bytes: &[u8]) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!("safe-keep-test-{}-{}", std::process::id(), name));
    let mut f = File::create(&path).expect("create temp file");
    f.write_all(bytes).expect("write temp file");
    f.flush().expect("flush temp file");
    path
  }

  fn payload(len: usize) -> Vec<u8> {
    (0..len).map(|i| (i * 31 % 251) as u8).collect()
  }

  #[test]
  fn stream_fingerprint_matches_file_hash() {
    let sizes = [0usize, 1, 100, 4095, 4096, 4097, 8192, 10_000, 1_000_003];
    // 不同的分块方式都必须得到同样的结果（模拟 4MB / 1MB / 64KB / 奇数块）
    let chunk_sizes = [64 * 1024usize, 1024 * 1024, 4 * 1024 * 1024, 7777];

    for &size in &sizes {
      let data = payload(size);
      let path = temp_file(&format!("{}.bin", size), &data);
      let expected = compute_head_tail_hash(&path).expect("file hash");

      for &chunk in &chunk_sizes {
        let mut fp = StreamFingerprint::new();
        for piece in data.chunks(chunk) {
          fp.update(piece);
        }
        assert_eq!(
          fp.finish(),
          expected,
          "size={} chunk={} 时的流式指纹与文件指纹不一致",
          size,
          chunk
        );
      }

      // 字节逐次喂入（极端分块）也必须一致
      let mut fp = StreamFingerprint::new();
      for b in &data {
        fp.update(std::slice::from_ref(b));
      }
      assert_eq!(fp.finish(), expected, "size={} 逐字节喂入时不一致", size);

      let _ = std::fs::remove_file(&path);
    }
  }

  #[test]
  fn stream_fingerprint_detects_tampering() {
    let data = payload(5000);
    let mut fp = StreamFingerprint::new();
    fp.update(&data);
    let fingerprint = fp.finish();

    let mut tampered = data.clone();
    tampered[10] ^= 0xFF;
    let path = temp_file("tampered.bin", &tampered);

    assert!(!verify_stream_against_dest(fingerprint, &path).expect("verify"));
    let _ = std::fs::remove_file(&path);
  }

  #[test]
  fn stream_fingerprint_accepts_identical_copy() {
    let data = payload(5000);
    let mut fp = StreamFingerprint::new();
    fp.update(&data);
    let path = temp_file("identical.bin", &data);
    assert!(verify_stream_against_dest(fp.finish(), &path).expect("verify"));
    let _ = std::fs::remove_file(&path);
  }
}
