mod header;

use crate::schema::Schema;
use crate::{
    offset::Offset,
    page::{self, Page},
};
use anyhow::Result;
use header::Header;
use std::{
    fs::File,
    io::{BufReader, Read, Seek},
};

pub trait SQL {
    fn schema(&mut self) -> Result<Schema>;
    fn page_at(&mut self, idx: u64) -> Result<Page>;
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
}

impl SQL for SQLiteFile {
    fn schema(&mut self) -> Result<Schema> {
        parser::read_page(self, 0u64.into(), Header::size()).and_then(Schema::try_from)
    }

    fn page_at(&mut self, idx: u64) -> Result<Page> {
        let offset = idx * self.head.page_size() as u64;
        parser::read_page(self, offset.into(), 0)
    }
}

mod parser {
    use super::*;

    pub fn read_page(file: &mut SQLiteFile, adr: Offset, pad: usize) -> Result<Page> {
        file.io.seek(adr.into())?;
        let mut data = vec![0u8; file.head.page_size() as usize];
        file.io.read_exact(&mut data)?;
        page::parser::build(&data, pad)
    }
}
