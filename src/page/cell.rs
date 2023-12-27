use crate::{
    codec::varint,
    value::{self, Type, Value},
    Result,
};

#[derive(Debug, Clone)]
pub struct TableLeafCell {
    pub id: u64,
    pub len: u64,
    pub record: Record,
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
