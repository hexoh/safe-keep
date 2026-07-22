use std::path::Path;

const IMAGE_EXTENSIONS: &[&str] = &[
  "jpg", "jpeg", "png", "heic", "heif", "webp", "gif", "bmp", "tiff", "raw", "arw", "cr2", "nef",
];

const VIDEO_EXTENSIONS: &[&str] = &["mp4", "mov", "avi", "mkv", "mts", "m2ts"];

pub fn is_supported_file(path: &Path) -> bool {
  path
    .extension()
    .and_then(|e| e.to_str())
    .map(|e| {
      let e = e.to_lowercase();
      IMAGE_EXTENSIONS.contains(&e.as_str()) || VIDEO_EXTENSIONS.contains(&e.as_str())
    })
    .unwrap_or(false)
}

#[allow(dead_code)]
pub fn is_image_file(path: &Path) -> bool {
  path
    .extension()
    .and_then(|e| e.to_str())
    .map(|e| IMAGE_EXTENSIONS.contains(&e.to_lowercase().as_str()))
    .unwrap_or(false)
}

#[allow(dead_code)]
pub fn is_video_file(path: &Path) -> bool {
  path
    .extension()
    .and_then(|e| e.to_str())
    .map(|e| VIDEO_EXTENSIONS.contains(&e.to_lowercase().as_str()))
    .unwrap_or(false)
}
