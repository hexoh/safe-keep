use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use super::conflict::{resolve_restore_dest, RestoreConflictStrategy};
use super::{RestoreFile, RestoreResult};
use crate::device::profile::DeviceProfile;

/// 连续失败达到该阈值即判定目标设备失效，避免为每个剩余文件空转重试。
const FATAL_FAIL_STREAK: u32 = 4;
const MAX_ATTEMPTS: u32 = 3;
const RETRY_DELAY_MS: u64 = 500;

pub fn restore_files(
  files: Vec<RestoreFile>,
  target_root: &Path,
  strategy: RestoreConflictStrategy,
  profile: DeviceProfile,
  cancel: Arc<AtomicBool>,
  on_progress: impl Fn(u64, u64, u64, u64, &str) + Send + Sync,
) -> RestoreResult {
  let total_files = files.len() as u64;
  let total_bytes: u64 = files.iter().map(|f| f.file_size as u64).sum();
  let started_at = std::time::Instant::now();

  let mut succeeded = 0u64;
  let mut failed = 0u64;
  let mut restored_bytes = 0u64;
  let mut errors = Vec::new();
  let mut succeeded_file_ids = Vec::new();
  let mut fail_streak = 0u32;
  let mut aborted = false;

  for file in &files {
    if cancel.load(Ordering::SeqCst) {
      break;
    }
    if aborted {
      break;
    }

    let src = Path::new(&file.dest_path);
    let rel = Path::new(&file.relative_path);
    let dest = target_root.join(rel);

    if let Some(parent) = dest.parent() {
      if let Err(e) = fs::create_dir_all(parent) {
        let msg = format!("Failed to create directory {}: {}", parent.display(), e);
        errors.push(msg);
        failed += 1;
        on_progress(total_files, succeeded + failed, succeeded, failed, &file.file_name);
        continue;
      }
    }

    let (resolved_dest, action) = resolve_restore_dest(&dest, strategy);
    if let super::conflict::ConflictAction::Skip = action {
      succeeded += 1;
      succeeded_file_ids.push(file.file_id);
      fail_streak = 0;
      on_progress(total_files, succeeded + failed, succeeded, failed, &file.file_name);
      continue;
    }

    match copy_with_retry(src, &resolved_dest, profile.copy_buf_size, &cancel) {
      Ok(bytes) => {
        succeeded += 1;
        succeeded_file_ids.push(file.file_id);
        restored_bytes += bytes;
        fail_streak = 0;
      }
      Err(e) => {
        if e == "cancelled" {
          break;
        }
        let msg = format!("Failed to restore {}: {}", src.display(), e);
        errors.push(msg);
        failed += 1;
        fail_streak += 1;
        if fail_streak >= FATAL_FAIL_STREAK {
          aborted = true;
          errors.push(
            "Restore aborted: the target device stopped responding. Remaining files were skipped."
              .to_string(),
          );
        }
      }
    }

    on_progress(total_files, succeeded + failed, succeeded, failed, &file.file_name);
  }

  let duration = started_at.elapsed().as_secs_f64();

  RestoreResult {
    total_files,
    succeeded,
    failed,
    total_bytes,
    restored_bytes,
    duration_secs: duration,
    errors,
    aborted,
    succeeded_file_ids,
  }
}

fn copy_with_retry(
  src: &Path,
  dest: &Path,
  buf_size: usize,
  cancel: &AtomicBool,
) -> Result<u64, String> {
  let mut last_error = String::new();
  for attempt in 0..MAX_ATTEMPTS {
    if cancel.load(Ordering::SeqCst) {
      return Err("cancelled".to_string());
    }
    match copy_file(src, dest, buf_size, cancel) {
      Ok(bytes) => return Ok(bytes),
      Err(e) => {
        if e == "cancelled" {
          return Err(e);
        }
        last_error = e;
        if attempt + 1 < MAX_ATTEMPTS {
          thread_sleep(RETRY_DELAY_MS);
          // 重试前清掉半截文件，避免后续写入变成追加
          let _ = fs::remove_file(dest);
        }
      }
    }
  }
  Err(last_error)
}

fn thread_sleep(ms: u64) {
  std::thread::sleep(Duration::from_millis(ms));
}

fn copy_file(
  src: &Path,
  dest: &Path,
  buf_size: usize,
  cancel: &AtomicBool,
) -> Result<u64, String> {
  let mut src_file = fs::File::open(src).map_err(|e| e.to_string())?;
  let src_modified = src_file.metadata().ok().and_then(|m| m.modified().ok());
  let mut dest_file = fs::File::create(dest).map_err(|e| e.to_string())?;

  let mut buf = vec![0u8; buf_size];
  let mut total_read = 0u64;

  loop {
    if cancel.load(Ordering::SeqCst) {
      return Err("cancelled".to_string());
    }

    let n = src_file.read(&mut buf).map_err(|e| e.to_string())?;
    if n == 0 {
      break;
    }
    dest_file.write_all(&buf[..n]).map_err(|e| e.to_string())?;
    total_read += n as u64;
  }

  // 还原后保留原始修改时间，后续扫描才认得出"已还原的同一份文件"。
  if let Some(modified) = src_modified {
    let _ = dest_file.set_modified(modified);
  }

  Ok(total_read)
}

/// 目标设备是否适合并发写入。MTP 写入本身是单会话串行的，
/// 并发只会互相拖慢甚至报错，因此当前保持串行，仅按设备放大缓冲。
#[allow(dead_code)]
pub fn restore_concurrency(profile: DeviceProfile) -> usize {
  if profile.is_slow() {
    1
  } else {
    profile.max_concurrency.min(4)
  }
}
