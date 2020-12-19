// todo: add file_cache
mod chunk;
mod lru_cache;
mod no_cache;
// mod linked_list;

pub use chunk::*;
pub use lru_cache::*;
pub use no_cache::*;
// pub use linked_list::*;

use std::io::{Read, Seek, Write};
use std::path::Path;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

pub trait CacheManager: Send + Sync {
    fn open(&self, path: &Path, file_id: u32) -> Box<dyn Cache>;
}
pub trait Cache: Read + Write + Seek + Send + Sync{
    fn offset(&self) -> u64;
}

pub enum CacheManagerEnum {}

/// Cache layer of CyKV
/// the same chunk wouldn't be written by more than 1 threads
/// Bitcask guarantees whenever only one file would be written
/// Conclusion: At most 1 thread write the same chunk
/// proof:
/// assume there are 2 threads write the same chunk
/// writing happens when only the store() method is called, which is called when only the chunk is dropped and attach a new file
/// drop happens when CyStore instance is dropped, that only happens in only one thread
/// only situation:
/// 2 threads attach a new file,

pub const DEFAULT_CACHE_SIZE: usize = 100 << 20;

// 4KiB CHUNK
const CHUNK_SIZE_SHIFT: u64 = 12;
const CHUNK_SIZE: usize = 1 << CHUNK_SIZE_SHIFT;
