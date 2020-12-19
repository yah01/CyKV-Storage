use crate::cache::{Cache, CacheManager};
use std::fs::{File, OpenOptions};
use std::io::{Read, Result, Seek, SeekFrom, Write};
use std::path::Path;
use std::sync::Arc;

#[derive(Clone)]
pub struct NoCacheManager;

impl CacheManager for NoCacheManager {
    fn open(&self, path: &Path, file_id: u32) -> Box<dyn Cache> {
        Box::new(NoCache::new(path))
    }
}

pub struct NoCache {
    file: File,
    offset: u64,
}

impl NoCache {
    fn new(path: impl AsRef<Path>) -> Self {
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(path)
            .unwrap();

        Self { file, offset: 0 }
    }
}

impl Cache for NoCache {
    fn offset(&self) -> u64 {
        self.offset
    }
}

impl Read for NoCache {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let len = self.file.read(buf)?;
        self.offset += len as u64;
        Ok(len)
    }
}

impl Write for NoCache {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let len = self.file.write(buf)?;
        self.offset += len as u64;
        Ok(len)
    }

    fn flush(&mut self) -> Result<()> {
        self.file.flush()
    }
}

impl Seek for NoCache {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        self.offset = self.file.seek(pos)?;
        Ok(self.offset)
    }
}
