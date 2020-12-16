use crate::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Seek, SeekFrom};
use std::path::{Path, PathBuf};

pub(crate) fn log_path(dir: &Path, id: u32) -> PathBuf {
    dir.join(format!("{}.log", id))
}

pub(crate) fn read_log(path: &Path, keydir: &mut HashMap<String, LogIndex>) -> Result<u32> {
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
        let command: Command = bson::from_document(bson::Document::from_reader(&mut log_file)?)?;
        let new_pos = log_file.seek(SeekFrom::Current(0))?;

        let log_index = LogIndex::new(log_id, pos, new_pos - pos);
        pos = new_pos;

        match command {
            Command::Set { key, value } => {
                keydir.insert(key, log_index);
            }
            Command::Remove { key } => {
                keydir.remove(&key);
            }
        }
    }

    Ok(log_id)
}

// pub fn for_each_log(dir: &Path, handle: fn() ) -> Result<u32> {
// 	for entry in dir.read_dir()? {
// 		let entry = entry?;
// 		if let Some(ext) = entry.path().extension() {
// 			if ext == ".log" {
//
// 			}
// 		}
// 	}
// }
