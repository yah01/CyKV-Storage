// IO Buffer with offset

use std::io::{self, Seek, Write, SeekFrom};

pub struct BufWriter<W: Write + Seek> {
    inner: W,
    pub pos: u64,
}

impl<W: Write + Seek> BufWriter<W> {
    pub fn new(mut inner: W) -> io::Result<Self> {
	    let pos = inner.seek(SeekFrom::Current(0))?;
        Ok(Self { inner, pos })
    }
}

impl<W: Write + Seek> Write for BufWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let len = self.inner.write(buf)?;
        self.pos += len as u64;
        Ok(len)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}
