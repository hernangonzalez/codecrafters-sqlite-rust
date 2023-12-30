mod header;

use crate::args::{Columns, Condition};
use crate::offset::Offset;
use crate::page::{self, Page};
use crate::schema::{Descriptor, Schema};
use crate::value::Value;
use anyhow::Result;
use header::Header;
use itertools::Itertools;
use std::{
    fs::File,
    io::{BufReader, Read, Seek},
};

pub trait SQL {
    fn schema(&mut self) -> Result<Schema>;
    fn page_at(&mut self, idx: u64) -> Result<Page>;
    fn select(&mut self, table: String) -> Result<Table>;
}

pub struct Table {
    pub desc: Descriptor,
    pub root: Page,
}

#[derive(Debug)]
struct Filter(usize, Value);

impl Table {
    fn find_columns(&self, names: &[String]) -> Vec<usize> {
        let cols = self.desc.column_names();
        names
            .iter()
            .flat_map(|name| cols.iter().find_position(|c| *c == name))
            .map(|c| c.0)
            .collect_vec()
    }

    fn filter_from(&self, c: Condition) -> Option<Filter> {
        self.find_columns(&[c.name])
            .first()
            .copied()
            .map(|i| Filter(i, c.value))
    }

    pub fn select(&self, cols: &Columns, cond: Option<Condition>) -> Vec<Vec<Value>> {
        let filter = cond.map(|c| self.filter_from(c)).flatten();

        let cols = self.find_columns(cols.as_slice());
        let cells = self.root.cells();
        cells
            .iter()
            .filter(|cell| {
                let Some(ref filter) = filter else {
                    return true;
                };
                let Some(val) = cell.record.values.get(filter.0) else {
                    return true;
                };
                val == &filter.1
            })
            .map(|cell| {
                cols.iter()
                    .copied()
                    .flat_map(|i| cell.record.values.get(i))
                    .cloned()
                    .collect_vec()
            })
            .collect_vec()
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
}

impl SQL for SQLiteFile {
    fn schema(&mut self) -> Result<Schema> {
        parser::read_page(self, 0u64.into(), Header::size()).and_then(Schema::try_from)
    }

    fn page_at(&mut self, idx: u64) -> Result<Page> {
        let offset = idx * self.head.page_size() as u64;
        parser::read_page(self, offset.into(), 0)
    }

    fn select(&mut self, table: String) -> Result<Table> {
        let schema = self.schema()?;
        let desc = schema.table_named(&table)?.clone();
        let root = self.page_at(desc.id)?;
        Ok(Table { desc, root })
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
