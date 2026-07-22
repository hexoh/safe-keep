use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use super::conflict::{resolve_restore_dest, RestoreConflictStrategy};
use super::{RestoreFile, RestoreResult};

const COPY_BUF_SIZE: usize = 64 * 1024;

pub fn restore_files(
  files: Vec<RestoreFile>,
  target_root: &Path,
  strategy: RestoreConflictStrategy,
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

  for file in &files {
    if cancel.load(Ordering::SeqCst) {
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
    match action {
      super::conflict::ConflictAction::Skip => {
        succeeded += 1;
        on_progress(total_files, succeeded + failed, succeeded, failed, &file.file_name);
        continue;
      }
      _ => {}
    }

    match copy_file(src, &resolved_dest, &cancel) {
      Ok(bytes) => {
        succeeded += 1;
        restored_bytes += bytes;
      }
      Err(e) => {
        let msg = format!("Failed to restore {}: {}", src.display(), e);
        errors.push(msg);
        failed += 1;
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
  }
}

fn copy_file(src: &Path, dest: &Path, cancel: &AtomicBool) -> Result<u64, String> {
  let mut src_file = fs::File::open(src).map_err(|e| e.to_string())?;
  let mut dest_file = fs::File::create(dest).map_err(|e| e.to_string())?;

  let mut buf = vec![0u8; COPY_BUF_SIZE];
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

  Ok(total_read)
}
