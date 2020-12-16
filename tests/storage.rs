use cykv::*;
use tempfile::TempDir;
use std::path::{Path, PathBuf};

fn NoCacheStore(path: PathBuf) -> CyStore<NoCacheManager> {
	CyStore::open(path,NoCacheManager{})
}

#[test]
fn get_stored_value() -> Result<()> {
	let dir = TempDir::new()?.into_path();
	let store = NoCacheStore(dir.clone());

	store.set("key1".to_owned(), "value1".to_owned())?;
	store.set("key2".to_owned(), "value2".to_owned())?;

	assert_eq!(store.get("key1".to_owned())?, Some("value1".to_owned()));
	assert_eq!(store.get("key2".to_owned())?, Some("value2".to_owned()));

	// Open from disk again and check persistent data
	// drop(store);
	// let store = NoCacheStore(dir);
	// assert_eq!(store.get("key1".to_owned())?, Some("value1".to_owned()));
	// assert_eq!(store.get("key2".to_owned())?, Some("value2".to_owned()));

	Ok(())
}

// #[test]
// fn overwrite_value() -> Result<()> {
// }
//
// #[test]
// fn get_non_existent_value() -> Result<()> {}
//
// #[test]
// fn remove_non_existent_key() -> Result<()> {}
//
// #[test]
// fn remove_key() -> Result<()> {}
//
// #[test]
// fn compaction() -> Result<()> {}
//
// #[test]
// fn concurrent_set() -> Result<()> {}
//
// #[test]
// fn concurrent_get() -> Result<()> {}
