pub mod detect;
pub mod list;
pub mod profile;

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum DeviceType {
  /// Android 手机 MTP 连接
  MTP,
  /// U 盘、SD 卡读卡器
  RemovableDisk,
  /// 本地硬盘或 SSD
  LocalDisk,
  /// 无法识别
  Unknown,
}
