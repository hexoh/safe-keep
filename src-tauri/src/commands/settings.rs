use tauri::State;

use crate::database::queries;
use crate::database::Database;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct AppSettings {
  pub default_dest: String,
  pub concurrent_threads: i32,
  pub compare_strategy: String,
  pub conflict_strategy: String,
  pub delete_strategy: String,
  pub dry_run_default: bool,
  pub auto_cleanup_reminder: bool,
  pub language: String,
  pub theme: String,
  pub auto_update: bool,
  pub update_channel: String,
}

impl Default for AppSettings {
  fn default() -> Self {
    Self {
      default_dest: String::new(),
      concurrent_threads: 4,
      compare_strategy: "size_time".to_string(),
      conflict_strategy: "rename".to_string(),
      delete_strategy: "recycle".to_string(),
      dry_run_default: false,
      auto_cleanup_reminder: true,
      language: "zh-CN".to_string(),
      theme: "system".to_string(),
      auto_update: true,
      update_channel: "stable".to_string(),
    }
  }
}

impl AppSettings {
  fn from_map(map: &std::collections::HashMap<String, String>) -> Self {
    Self {
      default_dest: map.get("default_dest").cloned().unwrap_or_default(),
      concurrent_threads: map
        .get("concurrent_threads")
        .and_then(|v| v.parse().ok())
        .unwrap_or(4),
      compare_strategy: map
        .get("compare_strategy")
        .cloned()
        .unwrap_or_else(|| "size_time".to_string()),
      conflict_strategy: map
        .get("conflict_strategy")
        .cloned()
        .unwrap_or_else(|| "rename".to_string()),
      delete_strategy: map
        .get("delete_strategy")
        .cloned()
        .unwrap_or_else(|| "recycle".to_string()),
      dry_run_default: map
        .get("dry_run_default")
        .and_then(|v| v.parse().ok())
        .unwrap_or(false),
      auto_cleanup_reminder: map
        .get("auto_cleanup_reminder")
        .and_then(|v| v.parse().ok())
        .unwrap_or(true),
      language: map.get("language").cloned().unwrap_or_else(|| "zh-CN".to_string()),
      theme: map.get("theme").cloned().unwrap_or_else(|| "system".to_string()),
      auto_update: map
        .get("auto_update")
        .and_then(|v| v.parse().ok())
        .unwrap_or(true),
      update_channel: map
        .get("update_channel")
        .cloned()
        .unwrap_or_else(|| "stable".to_string()),
    }
  }

  fn to_map(&self) -> Vec<(&str, String)> {
    vec![
      ("default_dest", self.default_dest.clone()),
      ("concurrent_threads", self.concurrent_threads.to_string()),
      ("compare_strategy", self.compare_strategy.clone()),
      ("conflict_strategy", self.conflict_strategy.clone()),
      ("delete_strategy", self.delete_strategy.clone()),
      ("dry_run_default", self.dry_run_default.to_string()),
      ("auto_cleanup_reminder", self.auto_cleanup_reminder.to_string()),
      ("language", self.language.clone()),
      ("theme", self.theme.clone()),
      ("auto_update", self.auto_update.to_string()),
      ("update_channel", self.update_channel.clone()),
    ]
  }
}

#[tauri::command]
pub async fn get_settings(db: State<'_, Database>) -> Result<AppSettings, String> {
  let conn = db.conn.lock().map_err(|e| e.to_string())?;
  let mut map = std::collections::HashMap::new();
  let mut stmt = conn
    .prepare("SELECT key, value FROM settings")
    .map_err(|e| e.to_string())?;
  let rows = stmt
    .query_map([], |row| {
      Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })
    .map_err(|e| e.to_string())?;
  for row in rows {
    let (k, v) = row.map_err(|e| e.to_string())?;
    map.insert(k, v);
  }
  Ok(AppSettings::from_map(&map))
}

#[tauri::command]
pub async fn save_settings(
  settings: AppSettings,
  db: State<'_, Database>,
) -> Result<(), String> {
  let conn = db.conn.lock().map_err(|e| e.to_string())?;
  for (key, value) in settings.to_map() {
    queries::set_setting(&conn, key, &value)?;
  }
  Ok(())
}
