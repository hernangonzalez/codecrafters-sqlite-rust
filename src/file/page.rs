use anyhow::{anyhow, Result};
use std::io::Read;

#[derive(Debug, Clone, Copy)]
pub enum Kind {
    TableInterior,
    TableLeaf,
    IndexInterior,
    IndexLeaf,
}

impl Kind {
    fn is_interior(&self) -> bool {
        match self {
            Self::TableInterior | Self::IndexInterior => true,
            Self::TableLeaf | Self::IndexLeaf => false,
        }
    }
}

impl TryFrom<u8> for Kind {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            0x5 => Ok(Self::TableInterior),
            0xD => Ok(Self::TableLeaf),
            0x2 => Ok(Self::IndexInterior),
            0xA => Ok(Self::IndexLeaf),
            _ => Err(anyhow!("Unknown page kind")),
        }
    }
}

#[derive(Debug)]
pub struct Header {
    pub kind: Kind,
    pub free_block: u16,
    pub cell_count: u16,
    pub cell_content: u32,
    pub fragment_count: u8,
    pub right_leave: Option<u32>,
}

#[derive(Debug)]
pub struct Page {
    pub head: Header,
    _data: Box<[u8]>,
}

pub fn read(io: &mut impl Read, len: usize) -> Result<Page> {
    let mut buf = vec![0u8; len];
    io.read_exact(&mut buf)?;
    parser::parse(&buf)
}

mod parser {
    use super::*;
    use nom::combinator::{cond, map, map_res};
    use nom::number::complete::{be_u16, be_u32, u8};
    use nom::sequence::tuple;
    use nom::{IResult, Parser};

    pub fn parse(io: &[u8]) -> Result<Page> {
        let (io, head) = parse_header(io).map_err(|_| anyhow!("could not read page"))?;
        let _data = Box::<[u8]>::from(io);
        Ok(Page { head, _data })
    }

    fn parse_kind(io: &[u8]) -> IResult<&[u8], Kind> {
        map_res(u8, Kind::try_from).parse(io)
    }

    fn cell_content(s: u16) -> u32 {
        match s {
            0 => 65_536,
            _ => s as u32,
        }
    }

    fn parse_header(io: &[u8]) -> IResult<&[u8], Header> {
        let (io, kind) = parse_kind(io)?;
        let fields = (be_u16, be_u16, be_u16, u8, cond(kind.is_interior(), be_u32));
        map(tuple(fields), move |t| Header {
            kind,
            free_block: t.0,
            cell_count: t.1,
            cell_content: cell_content(t.2),
            fragment_count: t.3,
            right_leave: t.4,
        })
        .parse(io)
    }
}
