use anyhow::Result;
use std::{
    fs::File,
    io::{BufReader, Read},
};

pub struct Header([u8; 100]);

impl Header {
    fn new(chunk: [u8; 100]) -> Result<Self> {
        anyhow::ensure!(chunk.starts_with(b"SQLite format 3\0"));
        Ok(Self(chunk))
    }

    pub fn page_size(&self) -> u16 {
        u16::from_be_bytes([self.0[16], self.0[17]])
    }
}

pub struct SQLiteFile(BufReader<File>);

impl SQLiteFile {
    pub fn open_at(name: &str) -> Result<SQLiteFile> {
        let file = File::open(name)?;
        let io = BufReader::new(file);
        Ok(Self(io))
    }

    pub fn header(&mut self) -> Result<Header> {
        let mut buf = [0u8; 100];
        self.0.read_exact(&mut buf)?;
        Header::new(buf)
    }
}
