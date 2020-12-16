use crate::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};

use crate::cache::{Cache, CacheManager, NoCacheManager};
use crate::engine::KvEngine;

#[derive(Serialize, Deserialize, Debug)]
struct LogIndex {
    id: u32,
    command_pos: u64,
    len: u64,
}

#[derive(Serialize, Deserialize, Debug)]
enum Command {
    Set { key: String, value: String },
    Remove { key: String },
}

#[derive(Clone)]
pub struct CyStore<C: CacheManager> {
    dir: Arc<PathBuf>, // The directory of the cykv stores data.

    keydir: Arc<RwLock<HashMap<String, LogIndex>>>, // Map key to log index.
    log_id: Arc<u32>,

    cache_manager: Arc<C>,
    writer: Arc<Mutex<CyStoreWriter>>,
}

impl<C: CacheManager> CyStore<C> {
    // todo: finish
    pub fn open(path: PathBuf, cache_manager: C) -> Self {
        let keydir = Arc::new(RwLock::new(HashMap::new()));
        let log_id = Arc::new(1);
        let cache = cache_manager.open(log_path(&path, *log_id).as_path());
        let writer = CyStoreWriter {
            keydir: Arc::clone(&keydir),
            log_id: Arc::clone(&log_id),
            writer: cache,
        };

        Self {
            dir: Arc::new(path),
            keydir,
            log_id,
            cache_manager: Arc::new(cache_manager),
            writer: Arc::new(Mutex::new(writer)),
        }
    }

    fn log_path(&self, id: u32) -> PathBuf {
        self.dir.join(format!("{}.log", id))
    }

    fn read_command(&self, log_index: &LogIndex) -> Result<Command> {
        let mut cache = self
            .cache_manager
            .open(self.log_path(log_index.id).as_path());
        cache.seek(SeekFrom::Start(log_index.command_pos))?;

        let cmd: Command = bson::from_document(bson::Document::from_reader(&mut cache)?)?;

        Ok(cmd)
    }
}

impl<C: CacheManager> KvEngine for CyStore<C> {
    fn get(&self, key: String) -> Result<Option<String>> {
        match self.keydir.read().unwrap().get(&key) {
            Some(log_index) => {
                let cmd: Command = self.read_command(log_index)?;

                if let Command::Set { key, value } = cmd {
                    Ok(Some(value))
                } else {
                    Ok(None)
                }
            }

            None => Ok(None),
        }
    }

    fn set(&self, key: String, value: String) -> Result<()> {
        self.writer.lock().unwrap().set(key, value)
    }

    fn remove(&self, key: String) -> Result<()> {
        self.writer.lock().unwrap().remove(key)
    }
}

struct CyStoreWriter {
    keydir: Arc<RwLock<HashMap<String, LogIndex>>>,
    log_id: Arc<u32>,

    writer: Box<dyn Cache>, // Log file writer
}

impl CyStoreWriter {
    fn set(&mut self, key: String, value: String) -> Result<()> {
        let cmd = Command::Set {
            key: key.clone(),
            value,
        };

        let log_index = self.append_command(cmd)?;
        self.keydir.write().unwrap().insert(key, log_index);

        Ok(())
    }

    fn remove(&mut self, key: String) -> Result<()> {
        self.keydir.write().unwrap().remove(&key);

        let cmd = Command::Remove { key };
        self.append_command(cmd)?;

        Ok(())
    }

    fn append_command(&mut self, cmd: Command) -> Result<LogIndex> {
        let pos = self.writer.offset();
        bson::to_document(&cmd)?.to_writer(&mut *self.writer)?;
        let len = self.writer.offset() - pos;

        Ok(LogIndex {
            id: *self.log_id,
            command_pos: pos,
            len,
        })
    }
}
