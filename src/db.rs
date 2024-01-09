mod header;
mod table;

use crate::offset::Offset;
use crate::page::Page;
use crate::schema::Schema;
use anyhow::Result;
use header::Header;
use std::cell::RefCell;
use std::fs::File;
use std::io::{BufReader, Read, Seek};
use table::Table;

pub trait SQL {
    fn schema(&self) -> Result<Schema>;
    fn page_at(&self, idx: i64) -> Result<Page>;
    fn table(&self, name: &str) -> Result<Table>;
}

pub struct SQLiteFile {
    pub head: Header,
    io: RefCell<BufReader<File>>,
}

impl SQLiteFile {
    pub fn open_at(name: &str) -> Result<SQLiteFile> {
        let file = File::open(name)?;
        let mut io = RefCell::new(BufReader::new(file));
        let head = Header::read(io.get_mut())?;
        Ok(Self { head, io })
    }
}

impl SQL for SQLiteFile {
    fn schema(&self) -> Result<Schema> {
        decode::read_page(self, 0i64.into(), Header::size()).and_then(Schema::try_from)
    }

    fn page_at(&self, idx: i64) -> Result<Page> {
        let idx = idx - 1;
        let offset = idx * self.head.page_size() as i64;
        decode::read_page(self, offset.into(), 0)
    }

    fn table(&self, name: &str) -> Result<Table> {
        let schema = self.schema()?;
        let desc = schema.table_named(name)?.clone();
        let root = self.page_at(desc.root)?;
        Ok(Table::new(self, desc, root))
    }
}
mod decode {
    use super::*;
    use crate::page;

    pub fn read_page(db: &SQLiteFile, adr: Offset, pad: usize) -> Result<Page> {
        let file = &mut db.io.borrow_mut();
        file.seek(adr.into())?;
        let mut data = vec![0u8; db.head.page_size() as usize];
        file.read_exact(&mut data)?;
        page::decode::take_page(&data, pad)
    }
}
