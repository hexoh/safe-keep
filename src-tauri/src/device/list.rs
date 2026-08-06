use super::DeviceType;
use serde::Serialize;
use std::path::Path;
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DeviceVolume {
  pub path: String,
  pub name: String,
  pub device_type: DeviceType,
  pub total_space: u64,
  pub free_space: u64,
}

/// 枚举当前已挂载的磁盘卷
pub fn list_volumes() -> Vec<DeviceVolume> {
  mount_points()
    .into_iter()
    .filter_map(|root| {
      let path = root.to_string_lossy().to_string();
      let name = if path == "/" {
        system_volume_name()
      } else {
        root
          .file_name()
          .and_then(|n| n.to_str())
          .unwrap_or("")
          .to_string()
      };
      let device_type = platform_type(&path);
      let (total_space, free_space) = space_avail(&root);
      Some(DeviceVolume {
        path,
        name,
        device_type,
        total_space,
        free_space,
      })
    })
    .collect()
}

/// 通过 statvfs 获取卷的总容量与可用容量
#[cfg(target_os = "macos")]
fn space_avail(path: &Path) -> (u64, u64) {
  let cpath = match std::ffi::CString::new(path.as_os_str().as_encoded_bytes()) {
    Ok(p) => p,
    Err(_) => return (0, 0),
  };
  // macOS 的 struct statfs 使用 64 位块计数，避免大容量卷溢出（statvfs 的 f_blocks 为 32 位）
  let mut stat: libc::statfs = unsafe { std::mem::zeroed() };
  if unsafe { libc::statfs(cpath.as_ptr(), &mut stat) } != 0 {
    return (0, 0);
  }
  let bsize = stat.f_bsize as u64;
  let total = stat.f_blocks;
  let free = stat.f_bavail;
  (total.saturating_mul(bsize), free.saturating_mul(bsize))
}

#[cfg(target_os = "linux")]
fn space_avail(path: &Path) -> (u64, u64) {
  use std::os::unix::ffi::OsStrExt;
  let cpath = match std::ffi::CString::new(path.as_os_str().as_bytes()) {
    Ok(p) => p,
    Err(_) => return (0, 0),
  };
  let mut stat: libc::statvfs = unsafe { std::mem::zeroed() };
  if unsafe { libc::statvfs(cpath.as_ptr(), &mut stat) } != 0 {
    return (0, 0);
  }
  let bsize = stat.f_bsize as u64;
  let total = stat.f_blocks.saturating_mul(bsize);
  let free = stat.f_bavail.saturating_mul(bsize);
  (total, free)
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn space_avail(_path: &Path) -> (u64, u64) {
  (0, 0)
}

/// 后台线程监控挂载点变化，实时发出 `devices:changed` 事件
pub fn start_device_watcher(app: AppHandle) {
  std::thread::spawn(move || {
    let mut last = list_volumes();
    loop {
      std::thread::sleep(std::time::Duration::from_millis(500));
      let now = list_volumes();
      if now != last {
        let _ = app.emit("devices:changed", &now);
        last = now;
      }
    }
  });
}

/// 各平台挂载点根列表
#[cfg(target_os = "macos")]
fn mount_points() -> Vec<std::path::PathBuf> {
  let mut roots = vec![std::path::PathBuf::from("/")];
  // 记录根卷设备 ID，跳过与之相同的系统数据卷
  let root_fsid = fsid_of("/");
  if let Ok(entries) = std::fs::read_dir("/Volumes") {
    for entry in entries.flatten() {
      let name = entry.file_name().to_string_lossy().to_string();
      // 跳过系统卷，避免与根卷 "/" 重复 (如 "Macintosh HD" / "Mac HD - Data")
      if name.starts_with('.') {
        continue;
      }
      if let Some(fsid) = fsid_of(&entry.path().to_string_lossy()) {
        if Some(fsid) == root_fsid {
          continue;
        }
      }
      roots.push(entry.path());
    }
  }
  roots
}

/// 获取某路径所在文件系统的设备 ID (macOS f_mntfromname → 源设备)
#[cfg(target_os = "macos")]
fn fsid_of(path: &str) -> Option<String> {
  let cpath = std::ffi::CString::new(path).ok()?;
  let mut stat: libc::statfs = unsafe { std::mem::zeroed() };
  if unsafe { libc::statfs(cpath.as_ptr(), &mut stat) } != 0 {
    return None;
  }
  let ptr = stat.f_mntfromname.as_ptr() as *const u8;
  let slice = unsafe { std::slice::from_raw_parts(ptr, stat.f_mntfromname.len()) };
  let end = slice.iter().position(|&b| b == 0).unwrap_or(slice.len());
  Some(String::from_utf8_lossy(&slice[..end]).to_string())
}

/// 解析系统盘的真实卷名（从 /Volumes 中找到与根卷同源设备的条目）
#[cfg(target_os = "macos")]
fn system_volume_name() -> String {
  let root_fsid = fsid_of("/");
  if let Ok(entries) = std::fs::read_dir("/Volumes") {
    for entry in entries.flatten() {
      let name = entry.file_name().to_string_lossy().to_string();
      if name.starts_with('.') {
        continue;
      }
      if let Some(fsid) = fsid_of(&entry.path().to_string_lossy()) {
        if root_fsid.as_deref() == Some(fsid.as_str()) {
          return name;
        }
      }
    }
  }
  "Local Disk".to_string()
}

#[cfg(target_os = "macos")]
fn platform_type(mnt: &str) -> DeviceType {
  if mnt.starts_with("/Volumes/") {
    DeviceType::RemovableDisk
  } else {
    DeviceType::LocalDisk
  }
}

#[cfg(target_os = "linux")]
fn mount_points() -> Vec<std::path::PathBuf> {
  use std::collections::BTreeSet;
  let mut roots = BTreeSet::new();
  let Ok(content) = std::fs::read_to_string("/proc/mounts") else {
    return vec![];
  };
  for line in content.lines() {
    let cols: Vec<&str> = line.split_whitespace().collect();
    if cols.len() < 2 {
      continue;
    }
    let mount = cols[1];
    if mount.starts_with("/media/")
      || mount.starts_with("/mnt/")
      || mount.starts_with("/run/media/")
    {
      roots.insert(std::path::PathBuf::from(mount));
    }
  }
  roots.into_iter().collect()
}

#[cfg(target_os = "linux")]
fn platform_type(_mnt: &str) -> DeviceType {
  DeviceType::RemovableDisk
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn mount_points() -> Vec<std::path::PathBuf> {
  let mut roots = Vec::new();
  for letter in b'A'..=b'Z' {
    let drive = format!("{}:\\", letter as char);
    if Path::new(&drive).exists() {
      roots.push(Path::new(&drive).to_path_buf());
    }
  }
  roots
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn platform_type(_mnt: &str) -> DeviceType {
  DeviceType::LocalDisk
}