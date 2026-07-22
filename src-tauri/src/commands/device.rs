use crate::device::detect::detect_device_type;
use crate::device::DeviceType;

#[tauri::command]
pub async fn detect_device(path: String) -> DeviceType {
  detect_device_type(&path)
}
