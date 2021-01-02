use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

pub(crate) fn log_path(dir: &Path, id: u32) -> PathBuf {
    dir.join(format!("{}.log", id))
}

pub(crate) fn hash_string(s: &String) -> u64 {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

// pub fn for_each_log(dir: &Path, handle: fn() ) -> Result<u32> {
// 	for entry in dir.read_dir()? {
// 		let entry = entry?;
// 		if let Some(ext) = entry.path().extension() {
// 			if ext == ".log" {
//
// 			}
// 		}
// 	}
// }
