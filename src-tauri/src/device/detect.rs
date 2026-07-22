use super::DeviceType;
use std::path::Path;

/// 根据路径判断设备类型
pub fn detect_device_type(path: &str) -> DeviceType {
  let path = Path::new(path);

  // 跨平台路径特征检测
  if is_mtp_path(path) {
    return DeviceType::MTP;
  }
  if is_removable_path(path) {
    return DeviceType::RemovableDisk;
  }

  // 平台特定检测
  platform_detect(path)
}

/// MTP 设备路径特征
fn is_mtp_path(path: &Path) -> bool {
  let s = path.to_string_lossy();
  // Windows MTP 路径
  if s.starts_with("\\\\?\\") {
    return true;
  }
  // Linux GVFS MTP 路径
  if s.contains("/gvfs/") {
    return true;
  }
  false
}

/// 可移动磁盘路径特征（macOS /Volumes 下非系统卷）
fn is_removable_path(path: &Path) -> bool {
  let s = path.to_string_lossy();
  if cfg!(target_os = "macos") && s.starts_with("/Volumes/") && !is_system_volume(&s) {
    return true;
  }
  false
}

fn is_system_volume(path_str: &str) -> bool {
  path_str.starts_with("/System/Volumes")
    || path_str.starts_with("/Volumes/Recovery")
    || path_str.starts_with("/Volumes/VM")
}

/// macOS: 使用 statfs 检测文件系统类型
#[cfg(target_os = "macos")]
fn platform_detect(path: &Path) -> DeviceType {
  let cpath = match std::ffi::CString::new(path.as_os_str().as_encoded_bytes()) {
    Ok(p) => p,
    Err(_) => return DeviceType::Unknown,
  };

  let mut stat: libc::statfs = unsafe { std::mem::zeroed() };
  let ret = unsafe { libc::statfs(cpath.as_ptr(), &mut stat) };
  if ret != 0 {
    return DeviceType::Unknown;
  }

  let fstype = unsafe {
    let ptr = stat.f_fstypename.as_ptr() as *const u8;
    let len = stat.f_fstypename.len();
    let slice = std::slice::from_raw_parts(ptr, len);
    let end = slice.iter().position(|&b| b == 0).unwrap_or(len);
    String::from_utf8_lossy(&slice[..end]).to_string()
  };

  match fstype.as_str() {
    "mtmfs" | "fusefs" => DeviceType::MTP,
    _ if path.to_string_lossy().starts_with("/Volumes/") => DeviceType::RemovableDisk,
    _ => DeviceType::LocalDisk,
  }
}

/// Windows: 基于路径前缀的简单检测
#[cfg(target_os = "windows")]
fn platform_detect(path: &Path) -> DeviceType {
  let s = path.to_string_lossy();
  if s.starts_with("\\\\?\\") {
    return DeviceType::MTP;
  }
  // 默认视为本地磁盘
  DeviceType::LocalDisk
}

/// Linux: 检测 GVFS 挂载点
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn platform_detect(_path: &Path) -> DeviceType {
  let s = _path.to_string_lossy();
  if s.contains("/gvfs/") || (s.contains("/run/user") && s.contains("/gvfs")) {
    return DeviceType::MTP;
  }
  DeviceType::LocalDisk
}
