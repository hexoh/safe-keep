mod commands;
mod copier;
mod database;
mod deleter;
mod device;
mod hasher;
mod restorer;
mod scanner;
mod utils;

use commands::backup::{
  cancel_backup, get_backup_history, get_failed_files, pause_backup, resume_backup,
  retry_failed_backup, start_backup, BackupState,
};
use commands::cleanup::{
  cancel_cleanup, dry_run_cleanup, execute_cleanup, get_source_roots, CleanupState,
};
use commands::restore::{cancel_restore, get_restorable_files, start_restore, RestoreState};
use commands::device::detect_device;
use commands::export::write_text_file;
use commands::scan::{cancel_scan, start_scan, ScannerState};
use commands::settings::{get_settings, save_settings};
use database::Database;
use tauri::Manager;
use tauri_plugin_updater; // required for plugin registration

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .manage(ScannerState {
      scanner: std::sync::Mutex::new(None),
    })
    .manage(BackupState::new())
    .manage(CleanupState::new())
    .manage(RestoreState::new())
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_updater::Builder::new().build())
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
      cancel_backup,
      get_backup_history,
      dry_run_cleanup,
      execute_cleanup,
      cancel_cleanup,
      get_source_roots,
      get_failed_files,
      retry_failed_backup,
      get_restorable_files,
      start_restore,
      cancel_restore,
      get_settings,
      save_settings,
      detect_device,
      write_text_file,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
