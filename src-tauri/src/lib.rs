mod commands;
mod copier;
mod database;
mod hasher;
mod scanner;
mod utils;

use commands::backup::{
  cancel_backup, pause_backup, resume_backup, start_backup, BackupState,
};
use commands::scan::{cancel_scan, start_scan, ScannerState};
use database::Database;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .manage(ScannerState {
      scanner: std::sync::Mutex::new(None),
    })
    .manage(BackupState::new())
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }

      let app_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
      std::fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;
      let db_path = app_dir.join("safe-keep.db");
      let db = Database::new(&db_path).expect("Failed to initialize database");
      app.manage(db);

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      start_scan,
      cancel_scan,
      start_backup,
      pause_backup,
      resume_backup,
      cancel_backup
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
