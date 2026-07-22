use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;

use super::conflict::{resolve_dest_path, ConflictStrategy};
use super::progress::BackupProgressEmitter;
use super::{BackupFile, BackupResult, FailedFile};

const COPY_BUF_SIZE: usize = 64 * 1024; // 64KB buffer

pub struct FileCopier {
  concurrency: usize,
  conflict_strategy: ConflictStrategy,
}

impl FileCopier {
  pub fn new(concurrency: usize, conflict_strategy: ConflictStrategy) -> Self {
    Self {
      concurrency,
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

    let completed_files = Arc::new(std::sync::atomic::AtomicU64::new(0));
    let copied_bytes = Arc::new(std::sync::atomic::AtomicU64::new(0));
    let failed_count = Arc::new(std::sync::atomic::AtomicU64::new(0));
    let errors = Arc::new(std::sync::Mutex::new(Vec::new()));
    let failed_files = Arc::new(std::sync::Mutex::new(Vec::new()));

    let (tx, rx) = mpsc::channel::<(u64, u64, String, f64)>();

    // 进度报告线程
    let progress_handle = {
      let cancel = cancel.clone();
      thread::spawn(move || {
        loop {
          match rx.recv_timeout(std::time::Duration::from_millis(200)) {
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
        }
      })
    };

    // 使用固定数量工作线程处理文件
    let work_done = Arc::new(AtomicBool::new(false));
    let file_index = Arc::new(std::sync::atomic::AtomicU64::new(0));
    let mut handles = Vec::new();

    for _ in 0..self.concurrency {
      let files = files.clone();
      let dest_root = dest_root.to_path_buf();
      let conflict_strategy = self.conflict_strategy;
      let cancel = cancel.clone();
      let paused = paused.clone();
      let completed_files = completed_files.clone();
      let copied_bytes = copied_bytes.clone();
      let failed_count = failed_count.clone();
      let errors = errors.clone();
      let failed_files = failed_files.clone();
      let tx = tx.clone();
      let file_index = file_index.clone();
      let work_done = work_done.clone();

      let handle = thread::spawn(move || {
        loop {
          // 检查取消
          if cancel.load(Ordering::SeqCst) {
            break;
          }

          // 暂停等待
          while paused.load(Ordering::SeqCst) && !cancel.load(Ordering::SeqCst) {
            thread::sleep(std::time::Duration::from_millis(100));
          }

          if cancel.load(Ordering::SeqCst) {
            break;
          }

          let idx = file_index.fetch_add(1, Ordering::SeqCst) as usize;
          if idx >= files.len() {
            break;
          }

          let file = &files[idx];
          let src = Path::new(&file.source_path);
          let rel = Path::new(&file.relative_path);
          let dest = dest_root.join(rel);

          // 创建父目录
          if let Some(parent) = dest.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
              let msg = format!("Failed to create directory {}: {}", parent.display(), e);
              errors.lock().unwrap().push(msg);
              failed_files.lock().unwrap().push(FailedFile {
                source_path: file.source_path.clone(),
                relative_path: file.relative_path.clone(),
                file_size: file.file_size,
                error: e.to_string(),
              });
              failed_count.fetch_add(1, Ordering::SeqCst);
              let _ = tx.send((0, 0, String::new(), 0.0));
              continue;
            }
          }

          // 冲突处理
          let (resolved_dest, action) = resolve_dest_path(&dest, conflict_strategy);
          match action {
            super::conflict::ConflictAction::Skip => {
              // 文件已存在且策略为跳过
              let _ = tx.send((
                completed_files.fetch_add(1, Ordering::SeqCst) + 1,
                copied_bytes.fetch_add(file.file_size, Ordering::SeqCst) + file.file_size,
                file.source_path.clone(),
                1.0,
              ));
              continue;
            }
            _ => {}
          }

          // 执行复制
          match copy_file(src, &resolved_dest, &cancel, &paused) {
            Ok(bytes_copied) => {
              let completed = completed_files.fetch_add(1, Ordering::SeqCst) + 1;
              let total_copied = copied_bytes.fetch_add(bytes_copied, Ordering::SeqCst) + bytes_copied;
              let _ = tx.send((
                completed,
                total_copied,
                Path::new(&file.source_path).file_name().and_then(|n| n.to_str()).unwrap_or("?").to_string(),
                1.0,
              ));
            }
            Err(e) => {
              let msg = format!("Failed to copy {}: {}", src.display(), e);
              errors.lock().unwrap().push(msg);
              failed_files.lock().unwrap().push(FailedFile {
                source_path: file.source_path.clone(),
                relative_path: file.relative_path.clone(),
                file_size: file.file_size,
                error: e.to_string(),
              });
              failed_count.fetch_add(1, Ordering::SeqCst);
              let _ = tx.send((0, 0, String::new(), 0.0));
            }
          }
        }

        work_done.store(true, Ordering::SeqCst);
      });

      handles.push(handle);
    }

    // 等待所有线程完成
    for h in handles {
      let _ = h.join();
    }

    drop(tx);
    let _ = progress_handle.join();

    let succeeded = total_files - failed_count.load(Ordering::SeqCst);
    let final_copied = copied_bytes.load(Ordering::SeqCst);
    let duration = started_at.elapsed().as_secs_f64();
    let avg_speed = if duration > 0.0 {
      (final_copied as f64 / 1024.0 / 1024.0) / duration
    } else {
      0.0
    };

    let error_list = errors.lock().unwrap().clone();
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
    })
  }
}

fn copy_file(
  src: &Path,
  dest: &Path,
  cancel: &AtomicBool,
  paused: &AtomicBool,
) -> Result<u64, String> {
  let mut src_file = fs::File::open(src).map_err(|e| e.to_string())?;
  let _src_len = src_file.metadata().map_err(|e| e.to_string())?.len();

  let mut dest_file = fs::File::create(dest).map_err(|e| e.to_string())?;

  let mut buf = vec![0u8; COPY_BUF_SIZE];
  let mut total_read = 0u64;

  loop {
    if cancel.load(Ordering::SeqCst) {
      return Err("cancelled".to_string());
    }
    while paused.load(Ordering::SeqCst) && !cancel.load(Ordering::SeqCst) {
      thread::sleep(std::time::Duration::from_millis(100));
    }

    let n = src_file.read(&mut buf).map_err(|e| e.to_string())?;
    if n == 0 {
      break;
    }
    dest_file.write_all(&buf[..n]).map_err(|e| e.to_string())?;
    total_read += n as u64;
  }

  Ok(total_read)
}
