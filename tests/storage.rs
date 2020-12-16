use cykv::*;
use std::path::PathBuf;
use std::sync::{Arc, Barrier};
use std::thread;
use tempfile::TempDir;
use walkdir::WalkDir;

fn no_cache_storage(path: PathBuf) -> Result<CyStore<NoCacheManager>> {
    CyStore::open(path, NoCacheManager {})
}

#[test]
fn get_stored_value() -> Result<()> {
    let dir = TempDir::new()?.into_path();
    let store = no_cache_storage(dir.clone())?;

    store.set("key1".to_owned(), "value1".to_owned())?;
    store.set("key2".to_owned(), "value2".to_owned())?;

    assert_eq!(store.get("key1".to_owned())?, Some("value1".to_owned()));
    assert_eq!(store.get("key2".to_owned())?, Some("value2".to_owned()));

    // Open from disk again and check persistent data
    drop(store);
    let store = no_cache_storage(dir)?;
    assert_eq!(store.get("key1".to_owned())?, Some("value1".to_owned()));
    assert_eq!(store.get("key2".to_owned())?, Some("value2".to_owned()));

    Ok(())
}

#[test]
fn overwrite_value() -> Result<()> {
    let temp_dir = TempDir::new()?.into_path();
    let store = no_cache_storage(temp_dir.clone())?;

    store.set("key1".to_owned(), "value1".to_owned())?;
    assert_eq!(store.get("key1".to_owned())?, Some("value1".to_owned()));
    store.set("key1".to_owned(), "value2".to_owned())?;
    assert_eq!(store.get("key1".to_owned())?, Some("value2".to_owned()));

    // Open from disk again and check persistent data
    drop(store);
    let store = no_cache_storage(temp_dir)?;
    assert_eq!(store.get("key1".to_owned())?, Some("value2".to_owned()));
    store.set("key1".to_owned(), "value3".to_owned())?;
    assert_eq!(store.get("key1".to_owned())?, Some("value3".to_owned()));

    Ok(())
}

#[test]
fn get_non_existent_value() -> Result<()> {
    let temp_dir = TempDir::new()?.into_path();
    let store = no_cache_storage(temp_dir.clone())?;

    store.set("key1".to_owned(), "value1".to_owned())?;
    assert_eq!(store.get("key2".to_owned())?, None);

    // Open from disk again and check persistent data
    drop(store);
    let store = no_cache_storage(temp_dir)?;
    assert_eq!(store.get("key2".to_owned())?, None);

    Ok(())
}

#[test]
fn remove_non_existent_key() -> Result<()> {
    let temp_dir = TempDir::new()?.into_path();
    let store = no_cache_storage(temp_dir)?;
    assert!(store.remove("key1".to_owned()).is_err());
    Ok(())
}

#[test]
fn remove_key() -> Result<()> {
    let temp_dir = TempDir::new()?.into_path();
    let store = no_cache_storage(temp_dir)?;
    store.set("key1".to_owned(), "value1".to_owned())?;
    assert!(store.remove("key1".to_owned()).is_ok());
    assert_eq!(store.get("key1".to_owned())?, None);
    Ok(())
}

// Insert data until total size of the directory decreases.
// Test data correctness after compaction.
#[test]
fn compaction() -> Result<()> {
    let temp_dir = TempDir::new()?.into_path();

    let dir_size = || {
        let entries = WalkDir::new(temp_dir.clone()).into_iter();
        let len: walkdir::Result<u64> = entries
            .map(|res| {
                res.and_then(|entry| entry.metadata())
                    .map(|metadata| metadata.len())
            })
            .sum();
        len.expect("fail to get directory size")
    };

    let mut current_size = dir_size();
    for iter in 0..1000 {
        let store = no_cache_storage(temp_dir.clone())?;

        for key_id in 0..1000 {
            let key = format!("key{}", key_id);
            let value = format!("{}", iter);
            store.set(key, value)?;
        }

        drop(store);

        let new_size = dir_size();
        if new_size > current_size {
            current_size = new_size;
            continue;
        }
        // Compaction triggered

        // reopen and check content
        let store = no_cache_storage(temp_dir.clone())?;
        for key_id in 0..1000 {
            let key = format!("key{}", key_id);
            assert_eq!(store.get(key)?, Some(format!("{}", iter)));
        }
        return Ok(());
    }

    panic!("No compaction detected");
}

#[test]
fn concurrent_set() -> Result<()> {
    let temp_dir = TempDir::new()?.into_path();
    let store = no_cache_storage(temp_dir.clone())?;
    let barrier = Arc::new(Barrier::new(1001));
    for i in 0..1000 {
        let store = store.clone();
        let barrier = barrier.clone();
        thread::spawn(move || {
            store
                .set(format!("key{}", i), format!("value{}", i))
                .unwrap();
            barrier.wait();
        });
    }
    barrier.wait();

    for i in 0..1000 {
        assert_eq!(store.get(format!("key{}", i))?, Some(format!("value{}", i)));
    }

    // Open from disk again and check persistent data
    drop(store);
    let store = no_cache_storage(temp_dir)?;
    for i in 0..1000 {
        assert_eq!(store.get(format!("key{}", i))?, Some(format!("value{}", i)));
    }

    Ok(())
}

#[test]
fn concurrent_get() -> Result<()> {
    let temp_dir = TempDir::new()?.into_path();
    let store = no_cache_storage(temp_dir.clone())?;
    for i in 0..100 {
        store
            .set(format!("key{}", i), format!("value{}", i))
            .unwrap();
    }

    let mut handles = Vec::new();
    for thread_id in 0..100 {
        let store = store.clone();
        let handle = thread::spawn(move || {
            for i in 0..100 {
                let key_id = (i + thread_id) % 100;
                assert_eq!(
                    store.get(format!("key{}", key_id)).unwrap(),
                    Some(format!("value{}", key_id))
                );
            }
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.join().unwrap();
    }

    // Open from disk again and check persistent data
    drop(store);
    let store = no_cache_storage(temp_dir)?;
    let mut handles = Vec::new();
    for thread_id in 0..100 {
        let store = store.clone();
        let handle = thread::spawn(move || {
            for i in 0..100 {
                let key_id = (i + thread_id) % 100;
                assert_eq!(
                    store.get(format!("key{}", key_id)).unwrap(),
                    Some(format!("value{}", key_id))
                );
            }
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.join().unwrap();
    }

    Ok(())
}
