use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use rayon::prelude::*;
use walkdir::WalkDir;

use super::filter::is_supported_file;
use super::progress::ScanProgressEmitter;
use super::{FileEntry, ScanResult};

pub struct FileScanner {
  cancelled: Arc<AtomicBool>,
}

impl FileScanner {
  pub fn new() -> Self {
    Self {
      cancelled: Arc::new(AtomicBool::new(false)),
    }
  }

  pub fn cancel(&self) {
    self.cancelled.store(true, Ordering::SeqCst);
  }

  pub fn scan(
    &self,
    root: &Path,
    progress: ScanProgressEmitter,
  ) -> Result<ScanResult, String> {
    let entries: Vec<_> = WalkDir::new(root)
      .follow_links(false)
      .same_file_system(false)
      .into_iter()
      .filter_map(|e| e.ok())
      .filter(|e| e.file_type().is_file())
      .filter(|e| {
        let name = e.file_name().to_string_lossy();
        !name.starts_with("._")
          && name != ".DS_Store"
          && !name.starts_with(".")
      })
      .filter(|e| is_supported_file(e.path()))
      .collect();

    let total = entries.len() as u64;
    let scanned = Arc::new(std::sync::atomic::AtomicU64::new(0));

    let files: Vec<FileEntry> = entries
      .par_iter()
      .filter_map(|entry| {
        if self.cancelled.load(Ordering::SeqCst) {
          return None;
        }

        let path = entry.path();
        let metadata = entry.metadata().ok()?;

        let scanned_count = scanned.fetch_add(1, Ordering::SeqCst) + 1;

        progress.emit( scanned_count, total, path.file_name()?.to_string_lossy().to_string(),
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
