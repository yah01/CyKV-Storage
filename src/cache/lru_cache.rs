use crate::cache::*;
use std::cmp::min;
use std::collections::HashMap;
use std::io;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::ops::Add;

pub struct LruCacheManager {
    cache_map: HashMap<PathBuf,Cache>
}

impl CacheManager for LruCacheManager {
    fn open(&self, path: &Path) -> Box<dyn Cache> {
        Box::new(
            LruCache::new(path,)
        )
    }
}

/// `Cache` is an abstraction of `File`
/// the cache caches the data in some chunks
/// each chunk has size `CHUNK_SIZE`
/// divide file data into multiple chunks: continuous `CHUNK_SIZE` bytes is a chunk
/// the i-th chunk has the `index i`
pub struct LruCache {
    path: PathBuf,

    list: Arc<ChunkList>,                 // shared chunk list
    chunk_map: HashMap<usize, Arc<Chunk>>, // map index to chunk

    cur_offset: u64,
}

impl LruCache {
    pub fn new(path: impl AsRef<Path>, file_id: u32, list: &Arc<ChunkList>) -> Self {
        let mut chunk_map = HashMap::new();

        Self {
            path: PathBuf::from(path.as_ref()),
            file_id,
            list: Arc::clone(list),
            chunk_map,
            cur_offset: 0,
        }
    }


    fn offset_to_index(offset: u64) -> usize {
        (offset >> CHUNK_SIZE_SHIFT) as usize
    }

    fn read_chunk(&mut self, index: usize, buf: &mut [u8]) -> io::Result<usize> {
        if let None = self.chunk_map.get(&index) {
            self.chunk_map.insert(index, self.list.get());
        }

        let mut chunk = self.chunk_map.get(&index).unwrap();

        // Need load data from disk
        if chunk.file_id != self.file_id || !chunk.has_file() {
            chunk.attach(&self.path, self.file_id, index)?;
        }

        return chunk.read(buf);
    }
}

impl Cache for LruCache {
    fn offset(&self) -> u64 {
        self.cur_offset
    }
}

impl Read for LruCache {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let (start, end) = (
            LruCache::offset_to_index(self.cur_offset),
            LruCache::offset_to_index(self.cur_offset + buf.len() as u64),
        );

        let mut len = 0;
        for i in start..=end {
            len += self.read_chunk(i, &mut buf[len..])?;
        }

        Ok(len)
    }
}

impl Write for LruCache {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        unimplemented!()
    }

    fn flush(&mut self) -> io::Result<()> {
        unimplemented!()
    }
}

impl Seek for LruCache {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        unimplemented!()
    }
}
