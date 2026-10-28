use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{AppHandle, Emitter, Manager, State};

use crate::database::compare;
use crate::database::Database;
use crate::device::profile::DeviceProfile;
use crate::scanner::progress::ScanProgressEmitter;
use crate::scanner::walk::FileScanner;
use crate::scanner::{FileEntry, ScanResult};

#[derive(serde::Deserialize)]
pub struct ScanOptions {
  pub source_path: String,
  #[allow(dead_code)]
  pub dest_path: Option<String>,
  pub include_images: Option<bool>,
  pub include_videos: Option<bool>,
  pub min_size: Option<u64>,
  pub max_size: Option<u64>,
  /// 增量模式：过滤掉数据库中已确认备份且未变更的文件。
  pub exclude_backed_up: Option<bool>,
}

/// 只持有取消标志，不持有 scanner 本身。
///
/// 这样真正的扫描可以扔进 `spawn_blocking`（慢速设备上一次扫描可能几分钟），
/// 不再占用 async 运行时的工作线程；命令层只需通过标志请求中止。
#[allow(dead_code)]
pub struct ScannerState {
  pub cancel: std::sync::Mutex<Option<Arc<AtomicBool>>>,
}

impl Default for ScannerState {
  fn default() -> Self {
    Self {
      cancel: std::sync::Mutex::new(None),
    }
  }
}

/// 首页统计任务的可取消标志，切换源路径时用于立即终止上一次遍历
#[allow(dead_code)]
pub struct CountState {
  pub cancel: std::sync::Mutex<Option<Arc<AtomicBool>>>,
}

/// 统计目录下媒体文件数量与大小（用于首页提示，不执行完整对比）
///
/// 异步 + spawn_blocking，避免在后台遍历大目录时阻塞主线程导致卡死。
/// 新启动的统计会先取消上一次的遍历（若有），并在循环中检查取消标志。
#[tauri::command]
pub async fn count_files(path: String, state: State<'_, CountState>) -> Result<(u64, u64), String> {
  let flag = Arc::new(AtomicBool::new(false));
  {
    let mut guard = state.cancel.lock().map_err(|e| e.to_string())?;
    if let Some(prev) = guard.as_ref() {
      prev.store(true, Ordering::Relaxed);
    }
    *guard = Some(flag.clone());
  }

  let flag_scan = flag.clone();
  let task = tauri::async_runtime::spawn_blocking(move || {
    let root = std::path::Path::new(&path);
    if !root.is_dir() {
      return (0, 0);
    }
    let mut count = 0u64;
    let mut bytes = 0u64;
    for entry in walkdir::WalkDir::new(root)
      .follow_links(false)
      .into_iter()
      .filter_entry(|e| {
        if e.depth() == 0 {
          return true;
        }
        let name = e.file_name().to_string_lossy();
        if e.file_type().is_dir() {
          !crate::scanner::filter::is_ignored_dir(&name)
        } else {
          !crate::scanner::filter::is_ignored_file(&name)
        }
      })
      .filter_map(|e| e.ok())
    {
      if flag_scan.load(Ordering::Relaxed) {
        break;
      }
      if entry.file_type().is_file()
        && crate::scanner::filter::is_supported_file(entry.path())
      {
        count += 1;
        if let Ok(m) = entry.metadata() {
          bytes += m.len();
        }
      }
    }
    (count, bytes)
  });

  let res = task.await.map_err(|e| e.to_string())?;

  {
    let mut guard = state.cancel.lock().map_err(|e| e.to_string())?;
    if guard.as_ref().is_some_and(|f| Arc::ptr_eq(f, &flag)) {
      *guard = None;
    }
  }

  Ok(res)
}

#[tauri::command]
pub fn cancel_count(state: State<'_, CountState>) -> Result<(), String> {
  let guard = state.cancel.lock().map_err(|e| e.to_string())?;
  if let Some(flag) = guard.as_ref() {
    flag.store(true, Ordering::Relaxed);
  }
  Ok(())
}

#[tauri::command]
pub async fn start_scan(
  app: AppHandle,
  options: ScanOptions,
  state: State<'_, ScannerState>,
) -> Result<ScanResult, String> {
  let path = Path::new(&options.source_path);
  if !path.exists() {
    return Err(format!("Path does not exist: {}", options.source_path));
  }
  if !path.is_dir() {
    return Err(format!("Path is not a directory: {}", options.source_path));
  }

  let source_root = path
    .canonicalize()
    .map_err(|e| e.to_string())?
    .to_string_lossy()
    .to_string();

  // 设备类型决定比对策略：MTP / FUSE 设备上禁止逐文件内容哈希，
  // 否则比对阶段会退化成"把每个文件重新下载一遍"。
  let profile = DeviceProfile::detect(Path::new(&source_root));

  let cancel_flag = Arc::new(AtomicBool::new(false));
  {
    let mut guard = state.cancel.lock().map_err(|e| e.to_string())?;
    if let Some(prev) = guard.as_ref() {
      prev.store(true, Ordering::SeqCst);
    }
    *guard = Some(cancel_flag.clone());
  }

  let app_handle = app.clone();

  // 扫描是纯阻塞 I/O（慢速设备上可能几分钟），必须离开 async 工作线程。
  let joined = tauri::async_runtime::spawn_blocking(move || -> Result<ScanResult, String> {
    let progress = ScanProgressEmitter::new(Some(Arc::new(app_handle.clone())));
    let scanner = FileScanner::with_cancel(cancel_flag);
    let scanned = scanner.scan(Path::new(&options.source_path), progress)?;

    let mut files: Vec<FileEntry> = scanned.files;

    if let Some(min) = options.min_size {
      files.retain(|f| f.file_size >= min);
    }
    if let Some(max) = options.max_size {
      files.retain(|f| f.file_size <= max);
    }
    if options.include_images == Some(false) {
      files.retain(|f| !f.is_image);
    }
    if options.include_videos == Some(false) {
      files.retain(|f| f.is_image);
    }

    // 增量对比：与数据库记录比对，标记状态
    let db = app_handle.state::<Database>();
    let mut files = compare::compute_file_statuses(db.inner(), &source_root, profile, files)?;

    if options.exclude_backed_up == Some(true) {
      files.retain(|f| f.status != "backed_up");
    }

    let total_files = files.len() as u64;
    let total_size = files.iter().map(|f| f.file_size).sum();

    let new_count = files.iter().filter(|f| f.status == "new").count();
    let backed_up_count = files.iter().filter(|f| f.status == "backed_up").count();
    let changed_count = files.iter().filter(|f| f.status == "changed").count();

    app_handle
      .emit(
        "scan:complete",
        serde_json::json!({
          "total": total_files,
          "new": new_count,
          "backed_up": backed_up_count,
          "changed": changed_count
        }),
      )
      .map_err(|e| e.to_string())?;

    Ok(ScanResult {
      total_files,
      total_size,
      files,
    })
  });

  let result = joined.await.map_err(|e| e.to_string())?;

  {
    let mut guard = state.cancel.lock().map_err(|e| e.to_string())?;
    *guard = None;
  }

  result
}

#[tauri::command]
pub async fn cancel_scan(state: State<'_, ScannerState>) -> Result<(), String> {
  let guard = state.cancel.lock().map_err(|e| e.to_string())?;
  if let Some(flag) = guard.as_ref() {
    flag.store(true, Ordering::SeqCst);
  }
  Ok(())
}
