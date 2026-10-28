use super::DeviceType;
use std::path::Path;

/// 根据路径判断设备类型
pub fn detect_device_type(path: &str) -> DeviceType {
  let path = Path::new(path);

  // 跨平台路径特征检测
  if is_mtp_path(path) {
    return DeviceType::MTP;
  }

  // 平台特定检测（macOS / Linux 会通过 statfs 区分 MTP 与可移动磁盘）
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

  // macFUSE 4.x 注册的文件系统类型名是 `macfuse`，老版 osxfuse 是 `osxfuse`，
  // `fusefs` 是 macFUSE 2.x 的历史名。三者都是第三方 FUSE 桥接，
  // 在本应用的使用场景里基本等同于 MTP 手机桥接（Android File Transfer / MacDroid 等），
  // 因此统一按 MTP 处理：关掉逐文件内容哈希、按单会话限制并发、删除走永久删除。
  match fstype.as_str() {
    "mtmfs" | "fusefs" | "macfuse" | "macfuse4" | "osxfuse" => DeviceType::MTP,
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
  if s.starts_with("/media/") || s.starts_with("/mnt/") || s.starts_with("/run/media/") {
    return DeviceType::RemovableDisk;
  }
  DeviceType::LocalDisk
}
