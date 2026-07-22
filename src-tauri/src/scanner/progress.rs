use std::sync::Arc;

use tauri::{AppHandle, Emitter};

use super::ScanProgress;

#[derive(Clone)]
pub struct ScanProgressEmitter {
  app_handle: Option<Arc<AppHandle>>,
}

impl ScanProgressEmitter {
  pub fn new(app_handle: Option<Arc<AppHandle>>) -> Self {
    Self { app_handle }
  }

  pub fn emit(&self, scanned: u64, total: u64, current_file: String) {
    if let Some(handle) = &self.app_handle {
      let _ = handle.emit(
        "scan:progress",
        ScanProgress {
          scanned,
          total: Some(total),
          current_file,
        },
      );
    }
  }

  #[allow(dead_code)]
  pub fn emit_complete(&self, total: u64) {
    if let Some(handle) = &self.app_handle {
      let _ = handle.emit("scan:complete", serde_json::json!({ "total": total }));
    }
  }

  #[allow(dead_code)]
  pub fn emit_error(&self, error: &str) {
    if let Some(handle) = &self.app_handle {
      let _ = handle.emit("scan:error", serde_json::json!({ "error": error }));
    }
  }
}
