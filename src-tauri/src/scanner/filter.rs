use std::path::Path;

const IMAGE_EXTENSIONS: &[&str] = &[
  "jpg", "jpeg", "png", "heic", "heif", "webp", "gif", "bmp", "tiff", "raw", "dng", "avif", "arw",
  "cr2", "nef",
];

const VIDEO_EXTENSIONS: &[&str] = &[
  "mp4", "mov", "avi", "mkv", "mts", "m2ts", "m4v", "webm", "3gp",
];

/// 系统 / 临时目录，遍历时直接跳过，避免扫描回收站、索引目录等。
const IGNORED_DIRS: &[&str] = &[
  "$RECYCLE.BIN",
  "System Volume Information",
  ".Trashes",
  ".Spotlight-V100",
  ".fseventsd",
  ".TemporaryItems",
  ".DocumentRevisions-V100",
  "@eaDir",
];

/// 是否应忽略该目录（系统目录或隐藏目录）。
pub fn is_ignored_dir(name: &str) -> bool {
  name.starts_with('.')
    || IGNORED_DIRS
      .iter()
      .any(|ignored| name.eq_ignore_ascii_case(ignored))
}

/// 是否应忽略该文件（macOS 资源分叉、隐藏文件等）。
pub fn is_ignored_file(name: &str) -> bool {
  name.starts_with("._") || name == ".DS_Store" || name.starts_with('.')
}

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
