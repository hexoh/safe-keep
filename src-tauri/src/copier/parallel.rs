use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use super::conflict::{resolve_dest_path, ConflictAction, ConflictStrategy};
use super::progress::BackupProgressEmitter;
use super::{BackupFile, BackupResult, FailedFile};
use crate::device::profile::DeviceProfile;
use crate::hasher::fast_hash::{verify_stream_against_dest, StreamFingerprint};

/// 连续失败达到该阈值就判定为设备级故障（拔线 / 掉盘 / 挂载点失效）。
///
/// 没有这道闸门时，设备一断开，剩余每个文件都要走满 3 次重试 × 500ms，
/// 几千个文件就是几个小时的无效等待，而且全程占着备份状态不放。
const FATAL_FAIL_STREAK: u32 = 6;
const MAX_ATTEMPTS: u32 = 3;
const RETRY_DELAY_MS: u64 = 500;

pub struct FileCopier {
  profile: DeviceProfile,
  concurrency: usize,
  conflict_strategy: ConflictStrategy,
}

impl FileCopier {
  /// `concurrency` 会被收敛到设备能承受的上限（MTP 是单会话协议，并发过高会互相抢锁）。
  pub fn new(profile: DeviceProfile, concurrency: usize, conflict_strategy: ConflictStrategy) -> Self {
    Self {
      profile,
      concurrency: profile.clamp_concurrency(concurrency),
      conflict_strategy,
    }
  }

  pub fn run(
    &self,
    files: Vec<BackupFile>,
    dest_root: &Path,
    progress: BackupProgressEmitter,
    cancel: Arc<AtomicBool>,
    paused: Arc<AtomicBool>,
  ) -> Result<BackupResult, String> {
    let total_files = files.len() as u64;
    let total_bytes: u64 = files.iter().map(|f| f.file_size).sum();
    let started_at = std::time::Instant::now();

    // 按目标相对路径排序：同一目录的文件集中处理，慢速设备上能显著减少
    // 目录句柄反复开关的往返次数。
    let mut files = files;
    files.sort_by(|a, b| a.dest_relative().cmp(b.dest_relative()));
    let files = Arc::new(files);

    let completed_files = Arc::new(std::sync::atomic::AtomicU64::new(0));
    let copied_bytes = Arc::new(std::sync::atomic::AtomicU64::new(0));
    let failed_count = Arc::new(std::sync::atomic::AtomicU64::new(0));
    let fail_streak = Arc::new(AtomicU32::new(0));
    let fatal = Arc::new(AtomicBool::new(false));
    let errors = Arc::new(std::sync::Mutex::new(Vec::new()));
    let failed_files = Arc::new(std::sync::Mutex::new(Vec::new()));
    let succeeded_files = Arc::new(std::sync::Mutex::new(Vec::new()));

    let (tx, rx) = mpsc::channel::<(u64, u64, String, f64)>();

    // 进度报告线程
    let progress_handle = {
      let cancel = cancel.clone();
      thread::spawn(move || loop {
        match rx.recv_timeout(Duration::from_millis(200)) {
          Ok((completed, copied, file, file_progress)) => {
            progress.emit(completed, copied, file, file_progress);
          }
          Err(mpsc::RecvTimeoutError::Timeout) => {
            if cancel.load(Ordering::SeqCst) {
              break;
            }
          }
          Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
      })
    };

    let file_index = Arc::new(std::sync::atomic::AtomicU64::new(0));
    let mut handles = Vec::new();

    for _ in 0..self.concurrency {
      let files = files.clone();
      let dest_root = dest_root.to_path_buf();
      let profile = self.profile;
      let conflict_strategy = self.conflict_strategy;
      let cancel = cancel.clone();
      let paused = paused.clone();
      let completed_files = completed_files.clone();
      let copied_bytes = copied_bytes.clone();
      let failed_count = failed_count.clone();
      let fail_streak = fail_streak.clone();
      let fatal = fatal.clone();
      let errors = errors.clone();
      let failed_files = failed_files.clone();
      let succeeded_files = succeeded_files.clone();
      let tx = tx.clone();
      let file_index = file_index.clone();

      let handle = thread::spawn(move || {
        loop {
          if cancel.load(Ordering::SeqCst) || fatal.load(Ordering::SeqCst) {
            break;
          }

          // 暂停等待
          while paused.load(Ordering::SeqCst) && !cancel.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_millis(100));
          }

          if cancel.load(Ordering::SeqCst) || fatal.load(Ordering::SeqCst) {
            break;
          }

          let idx = file_index.fetch_add(1, Ordering::SeqCst) as usize;
          if idx >= files.len() {
            break;
          }

          let file = &files[idx];
          let src = Path::new(&file.source_path);
          let rel = Path::new(file.dest_relative());
          let dest = dest_root.join(rel);

          // 创建父目录
          if let Some(parent) = dest.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
              record_failure(
                &errors,
                &failed_files,
                &failed_count,
                &fail_streak,
                &fatal,
                FailedFile {
                  source_path: file.source_path.clone(),
                  relative_path: file.relative_path.clone(),
                  dest_path: Some(dest.to_string_lossy().to_string()),
                  file_size: file.file_size,
                  error: e.to_string(),
                  modified_at: file.modified_at,
                },
              );
              let _ = tx.send((
                completed_files.load(Ordering::SeqCst),
                copied_bytes.load(Ordering::SeqCst),
                file.source_path.clone(),
                0.0,
              ));
              continue;
            }
          }

          // 冲突处理
          let (resolved_dest, action) = resolve_dest_path(&dest, conflict_strategy);
          if let ConflictAction::Skip = action {
            // 文件已存在且策略为跳过
            succeeded_files.lock().unwrap().push(file.relative_path.clone());
            fail_streak.store(0, Ordering::SeqCst);
            let completed = completed_files.fetch_add(1, Ordering::SeqCst) + 1;
            let total_copied =
              copied_bytes.fetch_add(file.file_size, Ordering::SeqCst) + file.file_size;
            let _ = tx.send((completed, total_copied, display_name(&file.source_path), 1.0));
            continue;
          }

          // 执行复制：瞬时错误最多重试 3 次；连续失败过多则中止整个任务
          match copy_with_retry(src, &resolved_dest, profile, &cancel, &paused) {
            Ok(bytes_copied) => {
              fail_streak.store(0, Ordering::SeqCst);
              let completed = completed_files.fetch_add(1, Ordering::SeqCst) + 1;
              let total_copied =
                copied_bytes.fetch_add(bytes_copied, Ordering::SeqCst) + bytes_copied;
              succeeded_files.lock().unwrap().push(file.relative_path.clone());
              let _ = tx.send((completed, total_copied, display_name(&file.source_path), 1.0));
            }
            Err(e) => {
              if e == "cancelled" {
                break;
              }
              record_failure(
                &errors,
                &failed_files,
                &failed_count,
                &fail_streak,
                &fatal,
                FailedFile {
                  source_path: file.source_path.clone(),
                  relative_path: file.relative_path.clone(),
                  dest_path: Some(resolved_dest.to_string_lossy().to_string()),
                  file_size: file.file_size,
                  error: e,
                  modified_at: file.modified_at,
                },
              );
              let _ = tx.send((
                completed_files.load(Ordering::SeqCst),
                copied_bytes.load(Ordering::SeqCst),
                file.source_path.clone(),
                0.0,
              ));
            }
          }
        }
      });

      handles.push(handle);
    }

    for h in handles {
      let _ = h.join();
    }

    drop(tx);
    let _ = progress_handle.join();

    let succeeded_list = succeeded_files.lock().unwrap().clone();
    // 取消 / 中止时未处理的文件既不算成功也不算失败，必须按实际成功列表统计。
    let succeeded = succeeded_list.len() as u64;
    let final_copied = copied_bytes.load(Ordering::SeqCst);
    let duration = started_at.elapsed().as_secs_f64();
    let avg_speed = if duration > 0.0 {
      (final_copied as f64 / 1024.0 / 1024.0) / duration
    } else {
      0.0
    };

    let aborted = fatal.load(Ordering::SeqCst) && !cancel.load(Ordering::SeqCst);
    let mut error_list = errors.lock().unwrap().clone();
    if aborted {
      error_list.push(
        "Aborted: the source device stopped responding (disconnected or unmounted). \
         Remaining files were left pending and can be retried later."
          .to_string(),
      );
    }
    let failed_list = failed_files.lock().unwrap().clone();

    Ok(BackupResult {
      total_files,
      succeeded,
      failed: failed_count.load(Ordering::SeqCst),
      total_bytes,
      copied_bytes: final_copied,
      duration_secs: duration,
      avg_speed_mbps: avg_speed,
      errors: error_list,
      failed_files: failed_list,
      aborted,
      succeeded_files: succeeded_list,
    })
  }
}

fn display_name(path: &str) -> String {
  Path::new(path)
    .file_name()
    .and_then(|n| n.to_str())
    .unwrap_or("?")
    .to_string()
}

fn record_failure(
  errors: &std::sync::Mutex<Vec<String>>,
  failed_files: &std::sync::Mutex<Vec<FailedFile>>,
  failed_count: &std::sync::atomic::AtomicU64,
  fail_streak: &AtomicU32,
  fatal: &AtomicBool,
  failure: FailedFile,
) {
  errors
    .lock()
    .unwrap()
    .push(format!("Failed to copy {}: {}", failure.source_path, failure.error));
  failed_files.lock().unwrap().push(failure);
  failed_count.fetch_add(1, Ordering::SeqCst);

  let streak = fail_streak.fetch_add(1, Ordering::SeqCst) + 1;
  if streak >= FATAL_FAIL_STREAK {
    fatal.store(true, Ordering::SeqCst);
  }
}

/// 复制文件，瞬时错误（如文件被占用）最多重试 3 次，间隔 500ms。
fn copy_with_retry(
  src: &Path,
  dest: &Path,
  profile: DeviceProfile,
  cancel: &AtomicBool,
  paused: &AtomicBool,
) -> Result<u64, String> {
  let mut last_error = String::new();
  for attempt in 0..MAX_ATTEMPTS {
    if cancel.load(Ordering::SeqCst) {
      return Err("cancelled".to_string());
    }

    match copy_file(src, dest, profile, cancel, paused) {
      Ok(bytes) => return Ok(bytes),
      Err(e) => {
        if e == "cancelled" {
          return Err(e);
        }
        // 完整性校验失败不做重试：重试一样会得到坏数据，只会更慢。
        if e.starts_with("integrity") {
          return Err(e);
        }
        last_error = e;
        if attempt + 1 < MAX_ATTEMPTS {
          thread::sleep(Duration::from_millis(RETRY_DELAY_MS));
        }
      }
    }
  }

  Err(last_error)
}

/// 顺序复制并在同一遍读取中算出源端的头尾指纹，
/// 之后只读目标端（本地盘）即可完成校验 —— 不需要回读源文件。
fn copy_file(
  src: &Path,
  dest: &Path,
  profile: DeviceProfile,
  cancel: &AtomicBool,
  paused: &AtomicBool,
) -> Result<u64, String> {
  let mut src_file = fs::File::open(src).map_err(|e| e.to_string())?;
  let src_meta = src_file.metadata().map_err(|e| e.to_string())?;
  let src_modified = src_meta.modified().ok();

  let mut dest_file = fs::File::create(dest).map_err(|e| e.to_string())?;

  let mut buf = vec![0u8; profile.copy_buf_size];
  let mut total_read = 0u64;
  let mut fingerprint = StreamFingerprint::new();

  loop {
    if cancel.load(Ordering::SeqCst) {
      return Err("cancelled".to_string());
    }
    while paused.load(Ordering::SeqCst) && !cancel.load(Ordering::SeqCst) {
      thread::sleep(Duration::from_millis(100));
    }

    let n = src_file.read(&mut buf).map_err(|e| e.to_string())?;
    if n == 0 {
      break;
    }
    dest_file.write_all(&buf[..n]).map_err(|e| e.to_string())?;
    fingerprint.update(&buf[..n]);
    total_read += n as u64;
  }

  // 保留源文件修改时间，清理前的完整性校验依赖它。
  if let Some(modified) = src_modified {
    let _ = dest_file.set_modified(modified);
  }
  drop(dest_file);

  // 用"边读边算"的源端指纹校验目标文件，顺带发现 FUSE 层返回的截断数据。
  let fingerprint = fingerprint.finish();
  match verify_stream_against_dest(fingerprint, dest) {
    Ok(true) => {}
    Ok(false) => {
      let _ = fs::remove_file(dest);
      return Err(format!(
        "integrity verification failed after copying {} bytes",
        total_read
      ));
    }
    Err(e) => {
      let _ = fs::remove_file(dest);
      return Err(format!("integrity verification failed: {}", e));
    }
  }

  Ok(total_read)
}
