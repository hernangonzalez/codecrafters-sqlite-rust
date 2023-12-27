mod header;

use crate::offset::Offset;
use crate::page::{self, Cell, Page, TableLeafCell};
use anyhow::{Context, Result};
use header::Header;
use itertools::Itertools;
use std::io::Read;
use std::{
    fs::File,
    io::{BufReader, Seek},
};

pub trait SQL {
    fn schema(&mut self) -> Result<Page>;
    fn tables(&mut self) -> Result<Vec<TableLeafCell>>;
    fn table_named(&mut self, name: &str) -> Result<Page>;
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
    fn schema(&mut self) -> Result<Page> {
        parser::read_page(self, 0u64.into(), Header::size())
    }

    fn tables(&mut self) -> Result<Vec<TableLeafCell>> {
        let schema = self.schema()?;
        let cells = schema.cells();
        Ok(cells.iter().filter(|c| c.is_table()).cloned().collect_vec())
    }

    fn table_named(&mut self, name: &str) -> Result<Page> {
        let schema = self.schema()?;
        let cells = schema.cells();
        let cell = cells
            .iter()
            .filter(|c| c.is_table())
            .find(|c| c.record.values.get(1).map(|s| s.as_str()).flatten() == Some(name))
            .context("table cell")?;

        let page_id = cell.record.values[3].as_int().unwrap() as u64 - 1;
        self.page_at(page_id)
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
