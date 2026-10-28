use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use rayon::prelude::*;
use walkdir::WalkDir;

use super::filter::{is_ignored_dir, is_ignored_file, is_supported_file};
use super::progress::ScanProgressEmitter;
use super::{FileEntry, ScanResult};

/// 发现阶段每遍历多少个文件上报一次进度，避免 IPC 事件刷屏。
const DISCOVERY_EMIT_INTERVAL: u64 = 64;

fn is_ignored_entry(entry: &walkdir::DirEntry) -> bool {
  // 根目录本身永远保留，即使用户选择的是隐藏目录。
  if entry.depth() == 0 {
    return false;
  }
  let name = entry.file_name().to_string_lossy();
  if entry.file_type().is_dir() {
    is_ignored_dir(&name)
  } else {
    is_ignored_file(&name)
  }
}

pub struct FileScanner {
  cancelled: Arc<AtomicBool>,
}

impl FileScanner {
  pub fn new() -> Self {
    Self {
      cancelled: Arc::new(AtomicBool::new(false)),
    }
  }

  /// 使用外部提供的取消标志，便于命令层只保存标志而不持有 scanner。
  pub fn with_cancel(cancelled: Arc<AtomicBool>) -> Self {
    Self { cancelled }
  }

  pub fn scan(
    &self,
    root: &Path,
    progress: ScanProgressEmitter,
  ) -> Result<ScanResult, String> {
    // 目录遍历本身是串行的（readdir 无法并行），慢速设备上这段可能持续几分钟。
    // 因此在遍历过程中就要上报"已发现 N 个"，否则界面在整个最慢的阶段都是静止的。
    let mut discovered = 0u64;
    let mut next_emit = DISCOVERY_EMIT_INTERVAL;

    let entries: Vec<_> = WalkDir::new(root)
      .follow_links(false)
      .same_file_system(false)
      .into_iter()
      .filter_entry(|e| !is_ignored_entry(e))
      .filter_map(|e| e.ok())
      .filter(|e| e.file_type().is_file())
      .filter(|e| is_supported_file(e.path()))
      .take_while(|_| !self.cancelled.load(Ordering::SeqCst))
      .inspect(|entry| {
        discovered += 1;
        if discovered >= next_emit {
          next_emit = discovered + DISCOVERY_EMIT_INTERVAL;
          progress.emit_discovering(
            discovered,
            entry.file_name().to_string_lossy().to_string(),
          );
        }
      })
      .collect();

    if self.cancelled.load(Ordering::SeqCst) {
      return Err("cancelled".to_string());
    }

    let total = entries.len() as u64;
    let scanned = Arc::new(std::sync::atomic::AtomicU64::new(0));

    let files: Vec<FileEntry> = entries
      .par_iter()
      .filter_map(|entry| {
        if self.cancelled.load(Ordering::SeqCst) {
          return None;
        }

        let path = entry.path();
        // 慢速设备上这一次 metadata() 就是一次协议往返，整个扫描的成本主要在这里，
        // 因此复用 DirEntry 已缓存的 file_type，只保留这一次 stat。
        let metadata = entry.metadata().ok()?;
        if !metadata.is_file() {
          return None;
        }

        let scanned_count = scanned.fetch_add(1, Ordering::SeqCst) + 1;

        progress.emit(
          scanned_count,
          total,
          path.file_name()?.to_string_lossy().to_string(),
        );

        let relative = path
          .strip_prefix(root)
          .unwrap_or(path)
          .to_string_lossy()
          .to_string();

        let ext = path
          .extension()
          .and_then(|e| e.to_str())
          .unwrap_or("")
          .to_lowercase();

        let is_image = matches!(
          ext.as_str(),
          "jpg"
            | "jpeg"
            | "png"
            | "heic"
            | "heif"
            | "webp"
            | "gif"
            | "bmp"
            | "tiff"
            | "raw"
            | "dng"
            | "avif"
            | "arw"
            | "cr2"
            | "nef"
        );

        Some(FileEntry {
          path: path.to_string_lossy().to_string(),
          relative_path: relative,
          file_name: entry.file_name().to_string_lossy().to_string(),
          file_size: metadata.len(),
          modified_at: metadata
            .modified()
            .ok()
            .and_then(|t| {
              t.duration_since(std::time::UNIX_EPOCH)
                .ok()
                .map(|d| d.as_secs() as i64)
            })
            .unwrap_or(0),
          is_image,
          status: String::new(),
        })
      })
      .collect();

    if self.cancelled.load(Ordering::SeqCst) {
      return Err("cancelled".to_string());
    }

    let total_size = files.par_iter().map(|f| f.file_size).sum();

    Ok(ScanResult {
      total_files: files.len() as u64,
      total_size,
      files,
    })
  }
}

impl Default for FileScanner {
  fn default() -> Self {
    Self::new()
  }
}
