use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RestoreConflictStrategy {
  Overwrite,
  Skip,
  KeepBoth,
}

impl RestoreConflictStrategy {
  pub fn from_str(s: &str) -> Self {
    match s {
      "skip" => Self::Skip,
      "keep_both" => Self::KeepBoth,
      _ => Self::Overwrite,
    }
  }
}

pub enum ConflictAction {
  Proceed,
  Skip,
}

pub fn resolve_restore_dest(
  dest: &Path,
  strategy: RestoreConflictStrategy,
) -> (std::path::PathBuf, ConflictAction) {
  if !dest.exists() {
    return (dest.to_path_buf(), ConflictAction::Proceed);
  }

  match strategy {
    RestoreConflictStrategy::Overwrite => (dest.to_path_buf(), ConflictAction::Proceed),
    RestoreConflictStrategy::Skip => (dest.to_path_buf(), ConflictAction::Skip),
    RestoreConflictStrategy::KeepBoth => {
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
