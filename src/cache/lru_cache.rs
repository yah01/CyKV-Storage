use crate::cache::*;
use lru;
use std::cmp::{max, min};
use std::collections::HashMap;
use std::fs::{self, Metadata};
use std::io;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

// file_id and index
#[derive(Eq, Hash, Clone, Copy)]
struct CacheKey(u32, usize);

impl PartialEq for CacheKey {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}

pub struct LruCacheManager {
    chunk_cache: Arc<Mutex<lru::LruCache<CacheKey, Arc<Mutex<Chunk>>>>>,
}

impl LruCacheManager {
    pub fn new(cache_bytes: u64) -> Self {
        Self {
            chunk_cache: Arc::new(Mutex::new(lru::LruCache::new(
                (cache_bytes / CHUNK_SIZE as u64) as usize,
            ))),
        }
    }
}

impl CacheManager for LruCacheManager {
    fn open(&self, path: &Path, file_id: u32) -> Box<dyn Cache> {
        let len = fs::metadata(path).map_or(0, |metadata| metadata.len());
        Box::new(LruCache::new(path, &self.chunk_cache, file_id, len))
    }
}

/// `Cache` is an abstraction of `File`
/// the cache caches the data in some chunks
/// each chunk has size `CHUNK_SIZE`
/// divide file data into multiple chunks: continuous `CHUNK_SIZE` bytes is a chunk
/// the i-th chunk has the `index i`
pub struct LruCache {
    path: PathBuf,
    file_id: u32,
    len: u64,
    chunk_cache: Arc<Mutex<lru::LruCache<CacheKey, Arc<Mutex<Chunk>>>>>, // shared chunk list
    chunk_map: HashMap<usize, Arc<Mutex<Chunk>>>,                        // map index to chunk node
    cur_offset: u64,
}

impl LruCache {
    fn new(
        path: impl AsRef<Path>,
        allocator: &Arc<Mutex<lru::LruCache<CacheKey, Arc<Mutex<Chunk>>>>>,
        file_id: u32,
        len: u64,
    ) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            file_id,
            len,
            chunk_cache: Arc::clone(allocator),
            chunk_map: HashMap::new(),
            cur_offset: 0,
        }
    }

    fn offset_to_index(offset: u64) -> usize {
        (offset >> CHUNK_SIZE_SHIFT) as usize
    }

    fn read_chunk(&mut self, offset: u64, index: usize, buf: &mut [u8]) -> io::Result<usize> {
        if !self.chunk_map.contains_key(&index) {
            let mut cache_guard = self.chunk_cache.lock().unwrap();
            let key = CacheKey(self.file_id, index);
            if !cache_guard.contains(&key) {
                cache_guard.put(key, Arc::new(Mutex::new(Chunk::new())));
            }
            self.chunk_map
                .insert(index, cache_guard.get(&key).unwrap().clone());
        }

        let chunk = self.chunk_map.get(&index).unwrap();
        let mut chunk = chunk.lock().unwrap();
        // Need load data from disk
        if chunk.file_id != self.file_id || chunk.has_file() {
            chunk.attach(&self.path, self.file_id, index)?;
        }

        return chunk.read(buf, offset - (index * CHUNK_SIZE) as u64);
    }

    fn write_chunk(&mut self, offset: u64, index: usize, buf: &[u8]) -> io::Result<usize> {
        if !self.chunk_map.contains_key(&index) {
            let mut cache_guard = self.chunk_cache.lock().unwrap();
            let key = CacheKey(self.file_id, index);
            if !cache_guard.contains(&key) {
                cache_guard.put(key, Arc::new(Mutex::new(Chunk::new())));
            }
            self.chunk_map
                .insert(index, cache_guard.get(&key).unwrap().clone());
        }

        let chunk = self.chunk_map.get(&index).unwrap();
        let mut chunk = chunk.lock().unwrap();
        // Need load data from disk
        if chunk.file_id != self.file_id || chunk.has_file() {
            chunk.attach(&self.path, self.file_id, index)?;
        }

        let len = chunk.write(buf, offset - (index * CHUNK_SIZE) as u64)?;
        chunk.sync()?;
        Ok(len)
    }
}

impl Cache for LruCache {
    fn offset(&self) -> u64 {
        self.cur_offset
    }
}

impl Read for LruCache {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.cur_offset >= self.len {
            return Ok(0); // No more data
        }

        let (start, end) = (
            LruCache::offset_to_index(self.cur_offset),
            LruCache::offset_to_index(min(self.len, self.cur_offset + buf.len() as u64)),
        );

        let mut len = 0;
        len += self.read_chunk(self.cur_offset, start, &mut buf[len..])?;

        for i in start + 1..end {
            len += self.read_chunk((i * CHUNK_SIZE) as u64, i, &mut buf[len..])?;
        }

        len += self.read_chunk(
            min(self.len, self.cur_offset + buf.len() as u64),
            end,
            &mut buf[len..],
        )?;

        self.cur_offset += len as u64;

        Ok(len)
    }
}

impl Write for LruCache {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let (start, end) = (
            LruCache::offset_to_index(self.cur_offset),
            LruCache::offset_to_index(self.cur_offset + buf.len() as u64),
        );

        let mut len = 0;
        len += self.write_chunk(self.cur_offset, start, &buf[len..])?;
        for i in start + 1..end {
            len += self.write_chunk((i * CHUNK_SIZE) as u64, i, &buf[len..])?;
        }
        len += self.write_chunk(
            self.cur_offset + buf.len() as u64,
            end,
            &buf[len..],
        )?;

        self.cur_offset += len as u64;
        self.len = max(self.len, self.cur_offset);

        Ok(len)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Seek for LruCache {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.cur_offset = match pos {
            SeekFrom::Start(offset) => offset,
            SeekFrom::End(offset) => (self.len as i64 + offset) as u64,
            SeekFrom::Current(offset) => (self.cur_offset as i64 + offset) as u64,
        };

        Ok(self.cur_offset)
    }
}
