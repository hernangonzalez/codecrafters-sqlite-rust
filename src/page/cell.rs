use crate::{
    codec::varint,
    value::{self, Type, Value},
    Result,
};

const CELL_TYPE_TABLE: &str = "table";
const NAME_PREFIX_SQLITE: &str = "sqlite_";

pub trait Cell {
    fn is_table(&self) -> bool;
    fn is_internal(&self) -> bool;
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TableLeafCell {
    pub id: u64,
    pub len: u64,
    pub record: Record,
}

impl Cell for TableLeafCell {
    fn is_table(&self) -> bool {
        self.record.values.first().map(|v| v.as_str()).flatten() == Some(CELL_TYPE_TABLE)
    }

    fn is_internal(&self) -> bool {
        self.record
            .values
            .get(1)
            .map(|s| s.as_str())
            .flatten()
            .map(|s| s.starts_with(NAME_PREFIX_SQLITE))
            .unwrap_or(false)
    }
}

#[derive(Debug, Clone)]
pub struct Record {
    pub types: Vec<Type>,
    pub values: Vec<Value>,
}

pub mod parser {
    use super::*;

    pub fn build(io: &[u8]) -> Result<TableLeafCell> {
        let (io, len) = varint::decode(io)?;
        let (io, id) = varint::decode(io)?;
        let record = build_record(io)?;
        Ok(TableLeafCell { id, len, record })
    }

    fn build_types(io: &[u8]) -> Result<(&[u8], Type)> {
        let (io, val) = varint::decode(io)?;
        Ok((io, val.into()))
    }

    fn build_record(io: &[u8]) -> Result<Record> {
        let (inner, tsz) = varint::decode(io)?;
        let mut buf = &io[..tsz as usize];
        buf = &buf[io.len() - inner.len()..];

        let mut types = Vec::new();
        while !buf.is_empty() {
            let (io, kind) = build_types(buf)?;
            types.push(kind);
            buf = io;
        }

        let mut values = Vec::with_capacity(types.len());
        let mut io = &io[tsz as usize..];
        for t in types.iter() {
            let (buf, val) = value::parser::build(io, *t)?;
            io = buf;
            values.push(val);
        }

        Ok(Record { types, values })
    }
}
