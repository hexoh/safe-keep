#[allow(dead_code)]
pub fn format_file_size(size: u64) -> String {
  const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
  let mut size = size as f64;
  let mut unit_idx = 0;

  while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
    size /= 1024.0;
    unit_idx += 1;
  }

  format!("{:.2} {}", size, UNITS[unit_idx])
}

#[allow(dead_code)]
pub fn format_timestamp(secs: i64) -> String {
  let dt = chrono::DateTime::from_timestamp(secs, 0)
    .unwrap_or_default();
  dt.format("%Y-%m-%d %H:%M:%S").to_string()
}
