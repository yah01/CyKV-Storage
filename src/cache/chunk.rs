use crate::cache::CHUNK_SIZE;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

pub enum State {
    None = 0,
    Empty = 1 << 0,
    Dirty = 1 << 1,
}

pub struct ChunkState {
    state: u8,
}

impl ChunkState {
    pub fn with_state(state: State) -> Self {
        Self { state: state as u8 }
    }

    pub fn set(&mut self, state: State) {
        self.state |= state as u8;
    }

    pub fn assign(&mut self, state: State) {
        self.state = state as u8;
    }

    pub fn clear(&mut self, state: State) {
        self.state &= 0xff ^ state as u8;
    }

    pub fn clear_all(&mut self) {
        self.state = State::None as u8;
    }

    pub fn is_empty(&self) -> bool {
        self.state & State::Empty as u8 == State::Empty as u8
    }

    pub fn is_dirty(&self) -> bool {
        self.state & State::Dirty as u8 == State::Dirty as u8
    }
}

pub struct Chunk {
    buf: [u8; CHUNK_SIZE],
    len: usize,

    // Attached file
    file: Option<File>,

    pub file_id: u32,
    pub index: usize,

    // state of the chunk
    // from lowest to highest bit:
    // empty, dirty
    pub state: ChunkState,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            buf: [0; CHUNK_SIZE],
            len: 0,
            file: None,
            file_id: 0,
            index: 0,
            state: ChunkState::with_state(State::Empty),
        }
    }

    pub fn attach(&mut self, path: &Path, file_id: u32, index: usize) -> io::Result<()> {
        self.store()?;

        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(path)?;
        self.file_id = file_id;
        self.attach_file(file, index);

        Ok(())
    }

    pub fn has_file(&self) -> bool {
        self.file.is_some()
    }

    pub fn clear(&mut self) -> io::Result<()> {
        self.store()?;
        self.file = None;
        Ok(())
    }

    fn attach_file(&mut self, file: File, index: usize) {
        self.file = Some(file);
        self.index = index;
        // self.cur_offset = 0;
    }

    // Only read() may call load()
    fn load(&mut self) -> io::Result<usize> {
        match &mut self.file {
            Some(file) => {
                file.seek(SeekFrom::Start((self.index * CHUNK_SIZE) as u64))?;
                self.len = file.read(&mut self.buf)?;
            }
            None => {
                self.len = 0;
            }
        }

        Ok(self.len)
    }

    // Only attach() and drop() may call store()
    // means only when the chunk cache is swapped out
    // and the cache is dropped, the data is written to disk
    fn store(&mut self) -> io::Result<usize> {
        if !self.state.is_dirty() {
            return Ok(0);
        }

        match &mut self.file {
            Some(file) => {
                file.seek(SeekFrom::Start((self.index * CHUNK_SIZE) as u64))?;
                let len = file.write(&self.buf[..self.len])?;
                file.sync_data()?;

                self.state.clear(State::Dirty);
                Ok(len)
            }
            None => Ok(0),
        }
    }

    pub(crate) fn read(&mut self, buf: &mut [u8], offset: u64) -> io::Result<usize> {
        if self.state.is_empty() {
            self.load()?;
        }

        let mut buf = buf;
        let len = buf.write(&self.buf[offset as usize..self.len])?;

        Ok(len)
    }

    pub(crate) fn write(&mut self, buf: &[u8], offset: u64) -> io::Result<usize> {
        if buf.len() == 0 {
            return Ok(0);
        }

        let mut cnt = 0;
        let offset = offset as usize;
        for i in offset..CHUNK_SIZE {
            self.buf[i] = buf[i - offset as usize];
            cnt += 1;
            if i - offset + 1 == buf.len() {
                break;
            }
        }
        self.len = offset + cnt;

        self.state.set(State::Dirty);

        Ok(cnt)
    }

    pub(crate) fn sync(&mut self) -> io::Result<()> {
        self.store()?;
        Ok(())
    }
}

impl Drop for Chunk {
    fn drop(&mut self) {
        self.store();
    }
}

// pub struct ChunkAllocator {
//     list: LinkedList<Arc<Chunk>>,
// }
//
// impl ChunkAllocator {
//     pub fn new(size: usize) -> Self {
//         let mut list = LinkedList::new();
//         let mut chunk_num = (size / CHUNK_SIZE) as i32;
//         if size % CHUNK_SIZE > 0 {
//             chunk_num += 1;
//         }
//
//         for _ in 0..chunk_num {
//             list.push_back(Arc::new(Chunk::new()));
//         }
//
//         Self { list }
//     }
//
//     pub fn get(&mut self) -> Arc<LinkedListNode<Arc<Chunk>>> {
//         let mut chunk = self.list.pop_front().unwrap();
//         chunk.clear();
//         self.list.push_back(chunk);
//         let node = self.list.back_node();
//         Arc::new(*node.unwrap())
//     }
// }
