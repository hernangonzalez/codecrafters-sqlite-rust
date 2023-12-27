mod cell;
mod header;
mod kind;

use crate::offset::{self, Offset};
use anyhow::{anyhow, Result};
pub use cell::TableLeafCell;
pub use header::Header;

#[derive(Debug)]
pub struct Page {
    pub head: Header,
    cells: Vec<Offset>,
    data: Box<[u8]>,
}

impl Page {
    pub fn cells(&self) -> Vec<TableLeafCell> {
        self.cells
            .iter()
            .map(|o| &self.data[o.as_usize()..] as &[u8])
            .flat_map(cell::parser::build)
            .collect()
    }
}

pub mod parser {
    use super::*;
    use nom::multi::fill;
    use nom::{IResult, Parser};

    pub fn build(io: &[u8], pad: usize) -> Result<Page> {
        let (_, head, cells) = header::parser::build(&io[pad..])
            .and_then(|(io, header)| {
                parse_offsets(io, header.cell_count).map(|(io, cells)| (io, header, cells))
            })
            .map_err(|_| anyhow!("could not read page"))?;

        let data = Box::<[u8]>::from(io);
        Ok(Page { head, data, cells })
    }

    fn parse_offsets(data: &[u8], count: u16) -> IResult<&[u8], Vec<Offset>> {
        let mut vec = vec![Offset::default(); count as usize];
        let res = fill(offset::parser::build, vec.as_mut_slice()).parse(data)?;
        Ok((res.0, vec))
    }
}
