use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConflictStrategy {
  Rename,
  Skip,
  Overwrite,
}

impl ConflictStrategy {
  pub fn from_str(s: &str) -> Self {
    match s {
      "skip" => Self::Skip,
      "overwrite" => Self::Overwrite,
      _ => Self::Rename,
    }
  }
}

/// 处理目标路径的文件名冲突，返回最终应该使用的目标路径
pub fn resolve_dest_path(
  dest: &Path,
  strategy: ConflictStrategy,
) -> (std::path::PathBuf, ConflictAction) {
  if !dest.exists() {
    return (dest.to_path_buf(), ConflictAction::Proceed);
  }

  match strategy {
    ConflictStrategy::Skip => (dest.to_path_buf(), ConflictAction::Skip),
    ConflictStrategy::Overwrite => (dest.to_path_buf(), ConflictAction::Overwrite),
    ConflictStrategy::Rename => {
      let parent = dest.parent().unwrap_or(Path::new(""));
      let stem = dest.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
      let ext = dest.extension().and_then(|s| s.to_str()).unwrap_or("");

      for i in 1..100 {
        let new_name = if ext.is_empty() {
          format!("{}_{}", stem, i)
        } else {
          format!("{}_{}.{}", stem, i, ext)
        };
        let new_path = parent.join(&new_name);
        if !new_path.exists() {
          return (new_path, ConflictAction::Proceed);
        }
      }
      (dest.to_path_buf(), ConflictAction::Skip)
    }
  }
}

pub enum ConflictAction {
  Proceed,
  Skip,
  Overwrite,
}
