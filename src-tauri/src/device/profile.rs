use std::path::Path;

use super::detect::detect_device_type;
use super::DeviceType;

/// MTP 每次 GetPartialObject 都有固定的往返开销（协议串行 + 设备固件处理），
/// 缓冲区越小往返次数越多。4MB 对本地盘无副作用，在 MTP 上能显著减少往返。
pub const MTP_COPY_BUF: usize = 4 * 1024 * 1024;
/// 本地 / 可移动磁盘的顺序读带宽已经很高，1MB 足够，避免多线程时内存翻倍。
pub const DISK_COPY_BUF: usize = 1024 * 1024;

/// 按设备类型给出 I/O 策略参数。
///
/// 非 POSIX 设备（MTP / FUSE 挂载的手机）不适合做逐文件内容哈希：
/// 随机读会被退化成"从头丢弃式读取"，等于为校验 4KB 而传输整个文件。
/// 这类设备也不支持回收站，删除即永久。
#[derive(Debug, Clone, Copy)]
pub struct DeviceProfile {
  pub device_type: DeviceType,
  /// 单次读写缓冲区大小
  pub copy_buf_size: usize,
  /// 该设备能承受的最大并发任务数
  pub max_concurrency: usize,
  /// 是否允许逐文件内容哈希（头尾 xxHash / SHA256）
  pub allows_content_hash: bool,
  /// 是否支持"移至回收站"
  pub allows_trash: bool,
}

impl DeviceProfile {
  pub fn for_type(device_type: DeviceType) -> Self {
    match device_type {
      DeviceType::MTP => Self {
        device_type,
        copy_buf_size: MTP_COPY_BUF,
        // MTP 是单会话协议，多线程会在设备端串行化甚至互相抢锁
        max_concurrency: 2,
        allows_content_hash: false,
        allows_trash: false,
      },
      DeviceType::RemovableDisk => Self {
        device_type,
        copy_buf_size: DISK_COPY_BUF,
        max_concurrency: 4,
        allows_content_hash: true,
        allows_trash: true,
      },
      DeviceType::LocalDisk | DeviceType::Unknown => Self {
        device_type,
        copy_buf_size: DISK_COPY_BUF,
        max_concurrency: 8,
        allows_content_hash: true,
        allows_trash: true,
      },
    }
  }

  /// 探测某个路径所属设备的策略。
  pub fn detect(path: &Path) -> Self {
    Self::for_type(detect_device_type(&path.to_string_lossy()))
  }

  /// 是否为"慢速非 POSIX 设备"，用于决定是否上报额外提示。
  pub fn is_slow(&self) -> bool {
    matches!(self.device_type, DeviceType::MTP)
  }

  /// 把用户配置的并发数收敛到设备能承受的上限。
  pub fn clamp_concurrency(&self, requested: usize) -> usize {
    requested.clamp(1, self.max_concurrency)
  }

  /// 该设备上实际可用的比对策略：不支持内容哈希时退化为 size_time。
  pub fn resolve_compare_strategy(&self, requested: &str) -> String {
    if !self.allows_content_hash && requested != "size_time" {
      "size_time".to_string()
    } else {
      requested.to_string()
    }
  }
}
