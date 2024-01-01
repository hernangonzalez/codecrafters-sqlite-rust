use crate::codec::varint;
use crate::value::{Type, Value};
use crate::Result;
use anyhow::Context;

#[derive(Debug, Copy, Clone)]
pub enum Column {
    ID,
    Content(usize),
}

#[derive(Debug, Clone)]
pub struct Record {
    pub types: Vec<Type>,
    pub values: Vec<Value>,
}

#[derive(Debug, Clone)]
pub struct TableLeafCell {
    pub id: i64,
    pub len: i64,
    pub record: Record,
}

impl TableLeafCell {
    pub fn value(&self, col: &Column) -> Result<Value> {
        match col {
            Column::ID => Ok(Value::Int(self.id)),
            Column::Content(i) => self.record.values.get(*i).cloned().context("Invalid index"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TableInteriorCell {
    pub lhs: u32,
    pub row: i64,
}

pub mod decode {
    use super::*;
    use crate::value;
    use nom::number::complete::be_u32;
    use nom::sequence::tuple;
    use nom::{IResult, Parser};

    pub fn take_interior_cell(io: &[u8]) -> IResult<&[u8], TableInteriorCell> {
        tuple((be_u32, varint::take))
            .map(|t| TableInteriorCell { lhs: t.0, row: t.1 })
            .parse(io)
    }

    pub fn take_leaf_cell(io: &[u8]) -> IResult<&[u8], TableLeafCell> {
        let (io, len) = varint::take(io)?;
        let (io, id) = varint::take(io)?;
        let (io, record) = take_record(io)?;
        Ok((io, TableLeafCell { id, len, record }))
    }

    pub fn take_record(io: &[u8]) -> IResult<&[u8], Record> {
        let (inner, tsz) = varint::take(io)?;
        let mut buf = &io[..tsz as usize];
        buf = &buf[io.len() - inner.len()..];

        let mut types = Vec::new();
        while !buf.is_empty() {
            let (io, kind) = value::decode::take_type(buf)?;
            types.push(kind);
            buf = io;
        }

        let mut values = Vec::with_capacity(types.len());
        let mut io = &io[tsz as usize..];
        for t in types.iter() {
            let (buf, val) = value::decode::take_value(io, *t)?;
            io = buf;
            values.push(val);
        }

        let rec = Record { types, values };
        Ok((io, rec))
    }
}
