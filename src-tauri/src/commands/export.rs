#[tauri::command]
pub async fn write_text_file(path: String, content: String) -> Result<(), String> {
  std::fs::write(&path, &content).map_err(|e| format!("Failed to write file: {}", e))
}
