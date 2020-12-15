use std::path::{Path, PathBuf};

pub fn log_path(dir: &Path, id: u32) -> PathBuf {
	dir.join(format!("{}.log", id))
}