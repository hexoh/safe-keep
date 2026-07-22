mod commands;
mod scanner;
mod utils;

use commands::scan::{cancel_scan, start_scan, ScannerState};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .manage(ScannerState {
      scanner: std::sync::Mutex::new(None),
    })
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![start_scan, cancel_scan])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
