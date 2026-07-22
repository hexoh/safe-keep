pub mod detect;

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
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
