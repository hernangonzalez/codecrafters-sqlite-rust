mod header;
mod page;

use anyhow::Result;
use header::Header;
use page::Page;
use std::{
    fs::File,
    io::{BufReader, Seek, SeekFrom},
};

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
        self.page_at(Header::size())
    }

    fn page_at(&mut self, offset: u64) -> Result<Page> {
        self.io.seek(SeekFrom::Start(offset))?;
        page::read(&mut self.io, self.head.page_size() as usize)
    }
}
