// use crate::cache::CHUNK_SIZE;
// use std::collections::linked_list::LinkedList;
// use std::fs::{File, OpenOptions};
// use std::io;
// use std::io::{Read, Seek, SeekFrom, Write};
// use std::path::Path;
// use std::sync::Arc;
//
// enum State {
//     None = 0,
//     Empty = 1 << 0,
//     Dirty = 1 << 1,
// }
//
// pub(crate) struct ChunkState {
//     state: u8,
// }
//
// impl ChunkState {
//     pub fn with_state(state: State) -> Self {
//         Self { state: state as u8 }
//     }
//
//     pub fn set(&mut self, state: State) {
//         self.state |= state as u8;
//     }
//
//     pub fn assign(&mut self, state: State) {
//         self.state = state as u8;
//     }
//
//     pub fn clear(&mut self, state: State) {
//         self.state &= 0xff ^ state as u8;
//     }
//
//     pub fn clear_all(&mut self) {
//         self.state = State::None as u8;
//     }
//
//     pub fn is_empty(&self) -> bool {
//         self.state & State::Empty as u8 == State::Empty as u8
//     }
//
//     pub fn is_dirty(&self) -> bool {
//         self.state & State::Dirty as u8 == State::Dirty as u8
//     }
// }
//
// pub(crate) struct Chunk {
//     buf: [u8; CHUNK_SIZE],
//     cur_offset: u64,
//     len: usize,
//
//     // Attached file
//     file: Option<File>,
//
//     pub file_id: u32,
//     pub index: usize,
//
//     // state of the chunk
//     // from lowest to highest bit:
//     // empty, dirty
//     pub state: ChunkState,
// }
//
// impl Chunk {
//     pub fn new() -> Self {
//         Self {
//             buf: [0; CHUNK_SIZE],
//             cur_offset: 0,
//             len: 0,
//             file: None,
//             file_id: 0,
//             index: 0,
//             state: ChunkState::with_state(State::Empty),
//         }
//     }
//
//     // pub fn with_file(path: &Path, index: usize) -> io::Result<Self> {
//     //     let mut file = OpenOptions::new().read(true).write(true).open(path)?;
//     //     file.seek(SeekFrom::Start((index * CHUNK_SIZE) as u64))?;
//     //
//     //     Ok(Self {
//     //         buf: [0; CHUNK_SIZE],
//     //         cur_offset: 0,
//     //         len: 0,
//     //         file: Some(file),
//     //         file_id: 0,
//     //         index,
//     //         state: ChunkState::with_state(State::Empty),
//     //     })
//     // }
//
//     pub fn attach(&mut self, path: &Path, file_id: u32, index: usize) -> io::Result<()> {
//         self.store()?;
//
//         let file = OpenOptions::new().read(true).write(true).open(path)?;
//         self.file_id = file_id;
//         self.attach_file(file, index);
//
//         Ok(())
//     }
//
//     pub fn has_file(&self) -> bool {
//         self.file.is_some()
//     }
//
//     pub fn clear(&mut self) -> io::Result<()> {
//         self.store()?;
//         self.file = None;
//         Ok(())
//     }
//
//     fn attach_file(&mut self, file: File, index: usize) {
//         self.file = Some(file);
//         self.index = index;
//         self.cur_offset = 0;
//     }
//
//     // Only read() may call load()
//     fn load(&mut self) -> io::Result<usize> {
//         match &self.file {
//             Some(mut file) => {
//                 file.seek(SeekFrom::Start((self.index * CHUNK_SIZE) as u64))?;
//                 self.len = file.read(&mut self.buf)?;
//             }
//             None => {
//                 self.len = 0;
//             }
//         }
//
//         Ok(self.len)
//     }
//
//     // Only attach() and drop() may call store()
//     // means only when the chunk cache is swapped out
//     // and the cache is dropped, the data is written to disk
//     fn store(&mut self) -> io::Result<usize> {
//         if !self.state.is_dirty() {
//             return Ok(0);
//         }
//
//         match &self.file {
//             Some(mut file) => {
//                 file.seek(SeekFrom::Start((self.index * CHUNK_SIZE) as u64))?;
//                 let len = file.write(&self.buf[..self.len])?;
//                 file.flush()?;
//
//                 self.state.clear(State::Dirty);
//                 Ok(len)
//             }
//             None => Ok(0),
//         }
//     }
// }
//
// impl Read for Chunk {
//     fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
//         if self.state.is_empty() {
//             self.load()?;
//         }
//
//         let len = buf.write(&self.buf[self.cur_offset as usize..self.len])?;
//         self.cur_offset += len as u64;
//         Ok(len)
//     }
// }
//
// impl Write for Chunk {
//     fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
//         if self.state.is_empty() {
//             self.load()?;
//         }
//
//         // Copy data from buf to inner buffer
//         for (i, byte) in buf.iter().enumerate() {
//             self.buf[i] = *byte;
//         }
//
//         self.state.set(State::Dirty);
//
//         Ok(buf.len())
//     }
//
//     fn flush(&mut self) -> io::Result<()> {
//         self.store()?;
//         Ok(())
//     }
// }
//
// impl Seek for Chunk {
//     fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
//         if let SeekFrom::Start(offset) = pos {
//             offset
//         }
//     }
// }
//
// impl Drop for Chunk {
//     fn drop(&mut self) {
//         self.store();
//     }
// }
//
// pub struct ChunkList {
//     list: LinkedList<Chunk>,
// }
//
// impl ChunkList {
//     pub fn new(size: usize) -> Self {
//         let mut list = LinkedList::new();
//         let mut chunk_num = (size / CHUNK_SIZE) as i32;
//         if size % CHUNK_SIZE > 0 {
//             chunk_num += 1;
//         }
//         println!()
//
//         for _ in 0..chunk_num {
//             list.push_back(Arc::new(Chunk::new()));
//         }
//
//         Self { list }
//     }
//
//     pub fn get(&mut self) -> Arc<Chunk> {
//         let mut chunk = self.list.pop_front().unwrap();
//         chunk.clear();
//         self.list.push_back(chunk);
//         Arc::clone(&chunk)
//     }
// }
