use anyhow::Result;
use std::{
    fs::File,
    io::{BufReader, Read, Seek, SeekFrom},
};

const HEADER_SIZE: usize = 100;

pub struct Header([u8; HEADER_SIZE]);

impl Header {
    fn read(io: &mut impl Read) -> Result<Self> {
        let mut buf = [0u8; 100];
        io.read_exact(&mut buf)?;
        anyhow::ensure!(buf.starts_with(b"SQLite format 3\0"));
        Ok(Self(buf))
    }

    pub fn page_size(&self) -> u16 {
        u16::from_be_bytes([self.0[16], self.0[17]])
    }
}

pub struct Page(Box<[u8]>);

impl Page {
    fn read(io: &mut impl Read, len: usize) -> Result<Page> {
        let mut buf = vec![0u8; len];
        io.read_exact(&mut buf)?;
        Ok(Page(buf.into_boxed_slice()))
    }

    pub fn row_count(&self) -> u16 {
        u16::from_be_bytes([self.0[3], self.0[4]])
    }
}

pub struct SQLiteFile {
    pub head: Header,
    io: BufReader<File>,
}

impl SQLiteFile {
    pub fn open_at(name: &str) -> Result<SQLiteFile> {
        let file = File::open(name)?;
        let mut io = BufReader::new(file);
        let head = Header::read(&mut io)?;
        Ok(Self { head, io })
    }

    pub fn schema(&mut self) -> Result<Page> {
        self.io.seek(SeekFrom::Start(HEADER_SIZE as u64))?;
        Page::read(&mut self.io, self.head.page_size() as usize)
    }
}
