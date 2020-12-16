use super::buffer::BufWriter;
use crate::cache::{Cache, CacheManager};
use crate::engine::KvEngine;
use crate::*;
use serde::{Deserialize, Serialize};
use std::cmp::max;
use std::collections::HashMap;
use std::fs::{self,File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};

// 1MiB
const COMPACT_THRESHOLD: u64 = 1 << 20;

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct LogIndex {
    id: u32,
    command_pos: u64,
    len: u64,
}

impl LogIndex {
    pub fn new(id: u32, pos: u64, len: u64) -> Self {
        Self {
            id,
            command_pos: pos,
            len,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) enum Command {
    Set { key: String, value: String },
    Remove { key: String },
}

#[derive(Clone)]
pub struct CyStore<C: CacheManager> {
    dir: Arc<PathBuf>, // The directory of the cykv stores data.

    keydir: Arc<RwLock<HashMap<String, LogIndex>>>, // Map key to log index.
    log_id: Arc<u32>,

    cache_manager: Arc<C>,
    writer: Arc<Mutex<CyStoreWriter<C>>>,
}

impl<C: CacheManager> CyStore<C> {
    pub fn open(dir: PathBuf, cache_manager: C) -> Result<Self> {
        let mut keydir = HashMap::new();
        let mut log_id = 0;
        let mut uncompacted = 0;

        for entry in dir.read_dir()? {
            let entry = entry?;
            if let Some(ext) = entry.path().extension() {
                if ext == "log" {
                    log_id = max(
                        log_id,
                        CyStore::<C>::read_log(
                            entry.path().as_path(),
                            &mut keydir,
                            &mut uncompacted,
                        )?,
                    );
                }
            }
        }

        let keydir = Arc::new(RwLock::new(keydir));
        let log_id = Arc::new(log_id + 1);

        let cache = cache_manager.open(log_path(&dir, *log_id).as_path());
        let dir = Arc::new(dir);
        let cache_manager = Arc::new(cache_manager);
        let writer = CyStoreWriter {
            dir: Arc::clone(&dir),
            cache_manager: Arc::clone(&cache_manager),
            keydir: Arc::clone(&keydir),
            log_id: Arc::clone(&log_id),
            uncompacted,
            writer: cache,
        };

        Ok(Self {
            dir,
            keydir,
            log_id,
            cache_manager,
            writer: Arc::new(Mutex::new(writer)),
        })
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

    fn read_log(
        path: &Path,
        keydir: &mut HashMap<String, LogIndex>,
        uncompacted: &mut u64,
    ) -> Result<u32> {
        let mut log_file = File::open(path)?;
        let log_id = path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .trim_end_matches(".log")
            .parse()
            .map_err(|_| CyKvError::Internal)?;

        let mut pos = 0;
        let len = log_file.metadata()?.len();

        while pos < len {
            let command: Command =
                bson::from_document(bson::Document::from_reader(&mut log_file)?)?;
            let new_pos = log_file.seek(SeekFrom::Current(0))?;

            let log_index = LogIndex::new(log_id, pos, new_pos - pos);
            pos = new_pos;

            match command {
                Command::Set { key, value } => {
                    if let Some(log_index) = keydir.insert(key, log_index) {
                        *uncompacted += log_index.len;
                    }
                }
                Command::Remove { key } => {
                    if let Some(log_index) = keydir.remove(&key) {
                        *uncompacted += log_index.len;
                    }
                }
            }
        }

        Ok(log_id)
    }
}

impl<C: CacheManager> KvEngine for CyStore<C> {
    fn get(&self, key: String) -> Result<Option<String>> {
        match self.keydir.read().unwrap().get(&key) {
            Some(log_index) => {
                let cmd: Command = self.read_command(log_index)?;

                if let Command::Set { key: _, value } = cmd {
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

struct CyStoreWriter<C: CacheManager> {
    dir: Arc<PathBuf>,
    cache_manager: Arc<C>,
    keydir: Arc<RwLock<HashMap<String, LogIndex>>>,
    log_id: Arc<u32>,
    uncompacted: u64,
    writer: Box<dyn Cache>, // log file writer
}

impl<C: CacheManager> CyStoreWriter<C> {
    fn set(&mut self, key: String, value: String) -> Result<()> {
        let cmd = Command::Set {
            key: key.clone(),
            value,
        };

        let log_index = self.append_command(cmd)?;
        if let Some(log_index) = self.keydir.write().unwrap().insert(key, log_index) {
            self.uncompacted += log_index.len;
        }

        if self.uncompacted >= COMPACT_THRESHOLD {
            self.compact()?;
        }

        Ok(())
    }

    fn remove(&mut self, key: String) -> Result<()> {
        match self.keydir.write().unwrap().remove(&key) {
            Some(log_index) => self.uncompacted += log_index.len,
            None => return Err(CyKvError::KeyNotFound(key)),
        }

        let cmd = Command::Remove { key };
        self.uncompacted += self.append_command(cmd)?.len;

        if self.uncompacted >= COMPACT_THRESHOLD {
            self.compact()?;
        }

        Ok(())
    }

    fn append_command(&mut self, cmd: Command) -> Result<LogIndex> {
        let pos = self.writer.offset();
        bson::to_document(&cmd)?.to_writer(&mut *self.writer)?;
        let len = self.writer.offset() - pos;

        Ok(LogIndex::new(*self.log_id, pos, len))
    }

    fn read_command(&self, log_index: &LogIndex) -> Result<Command> {
        let mut cache = self
            .cache_manager
            .open(utils::log_path(self.dir.as_path(), log_index.id).as_path());
        cache.seek(SeekFrom::Start(log_index.command_pos))?;

        let cmd: Command = bson::from_document(bson::Document::from_reader(&mut cache)?)?;

        Ok(cmd)
    }

    fn compact(&mut self) -> Result<()> {
        let compact_log_id = *self.log_id + 1;
        let compaction_file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(utils::log_path(self.dir.as_path(), compact_log_id).as_path())?;
        let mut writer = BufWriter::new(compaction_file)?;

        let mut keydir = self.keydir.write().unwrap();
        for (key, log_index) in keydir.iter_mut() {
            let cmd = self.read_command(log_index)?;
            let pos = writer.pos;
            bson::to_document(&cmd)?.to_writer(&mut writer)?;

            log_index.id = compact_log_id;
            log_index.command_pos = pos;
            log_index.len = writer.pos - pos;
        }
        drop(keydir);
        drop(writer);

        // Remove old log files
        for entry in self.dir.read_dir()? {
            let entry = entry?;
            if let Some(ext) = entry.path().extension() {
                if ext == "log" {
                    let log_id: u32 = entry
                        .file_name()
                        .to_str()
                        .unwrap()
                        .trim_end_matches(".log")
                        .parse()
                        .map_err(|_| CyKvError::Internal)?;

                    if log_id < *self.log_id {
                        fs::remove_file(entry.path().as_path())?;
                    }
                }
            }
        }
        self.uncompacted = 0;

        // Rename the compaction file
        fs::rename(log_path(self.dir.as_path(),compact_log_id),
                        log_path(self.dir.as_path(),*self.log_id-1))?;

        Ok(())
    }
}
