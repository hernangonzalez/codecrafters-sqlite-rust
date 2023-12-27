mod header;

use crate::offset::Offset;
use crate::page::{self, Cell, Page, TableLeafCell};
use anyhow::Result;
use header::Header;
use itertools::Itertools;
use std::io::Read;
use std::{
    fs::File,
    io::{BufReader, Seek},
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
}

impl SQLiteFile {
    pub fn schema(&mut self) -> Result<Page> {
        self.read_page(0u64.into(), Header::size())
    }

    fn read_page(&mut self, adr: Offset, pad: usize) -> Result<Page> {
        self.io.seek(adr.into())?;
        let mut data = vec![0u8; self.head.page_size() as usize];
        self.io.read_exact(&mut data)?;
        page::parser::build(&data, pad)
    }
}

#[allow(unreachable_code)]
impl SQLiteFile {
    pub fn tables(&mut self) -> Result<Vec<TableLeafCell>> {
        let schema = self.schema()?;
        let cells = schema.cells();
        Ok(cells.iter().filter(|c| c.is_table()).cloned().collect_vec())
    }
}
