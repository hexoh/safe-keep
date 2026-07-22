use std::sync::Arc;
use std::time::Instant;

use tauri::{AppHandle, Emitter};

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CopyProgressPayload {
  pub total_files: u64,
  pub completed_files: u64,
  pub total_bytes: u64,
  pub copied_bytes: u64,
  pub current_file: String,
  pub current_file_progress: f64,
  pub speed_mbps: f64,
  pub remaining_secs: Option<u64>,
}

#[derive(Clone)]
pub struct BackupProgressEmitter {
  app_handle: Option<Arc<AppHandle>>,
  pub(crate) start_time: Instant,
  total_files: u64,
  total_bytes: u64,
}

impl BackupProgressEmitter {
  pub fn new(
    app_handle: Option<Arc<AppHandle>>,
    total_files: u64,
    total_bytes: u64,
  ) -> Self {
    Self {
      app_handle,
      start_time: Instant::now(),
      total_files,
      total_bytes,
    }
  }

  pub fn emit(
    &self,
    completed_files: u64,
    copied_bytes: u64,
    current_file: String,
    current_file_progress: f64,
  ) {
    if let Some(handle) = &self.app_handle {
      let elapsed = self.start_time.elapsed().as_secs_f64();
      let speed_mbps = if elapsed > 0.0 {
        (copied_bytes as f64 / 1024.0 / 1024.0) / elapsed
      } else {
        0.0
      };

      let remaining_secs = if speed_mbps > 0.0 {
        let remaining_bytes = self.total_bytes.saturating_sub(copied_bytes);
        Some((remaining_bytes as f64 / 1024.0 / 1024.0 / speed_mbps) as u64)
      } else {
        None
      };

      let _ = handle.emit(
        "backup:progress",
        CopyProgressPayload {
          total_files: self.total_files,
          completed_files,
          total_bytes: self.total_bytes,
          copied_bytes,
          current_file,
          current_file_progress,
          speed_mbps,
          remaining_secs,
        },
      );
    }
  }

  pub fn emit_log(&self, message: &str) {
    if let Some(handle) = &self.app_handle {
      let _ = handle.emit("backup:log", serde_json::json!({ "message": message }));
    }
  }
}
