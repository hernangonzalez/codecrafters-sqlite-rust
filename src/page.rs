mod cell;
mod header;
mod kind;

use crate::offset::{self, Offset};
pub use crate::page::kind::Kind;
use crate::Result;
use anyhow::{bail, ensure, Context};
pub use cell::{Column, TableInteriorCell, TableLeafCell};
pub use header::Header;

#[derive(Debug)]
pub struct Page {
    pub head: Header,
    cells: Vec<Offset>,
    data: Box<[u8]>,
}

impl Page {
    pub fn into_leaf(self) -> Result<TableLeafPage> {
        ensure!(self.head.kind == Kind::TableLeaf);
        Ok(TableLeafPage(self))
    }

    pub fn into_interior(self) -> Result<TableInteriorPage> {
        ensure!(self.head.kind == Kind::TableInterior);
        Ok(TableInteriorPage(self))
    }
}

pub struct TableLeafPage(Page);

impl TableLeafPage {
    pub fn cells(&self) -> Vec<TableLeafCell> {
        self.0
            .cells
            .iter()
            .map(|o| &self.0.data[o.as_usize()..] as &[u8])
            .flat_map(cell::decode::take_leaf_cell)
            .map(|r| r.1)
            .collect()
    }
}

pub struct TableInteriorPage(Page);

impl TableInteriorPage {
    pub fn cells(&self) -> Vec<TableInteriorCell> {
        self.0
            .cells
            .iter()
            .map(|o| &self.0.data[o.as_usize()..] as &[u8])
            .flat_map(cell::decode::take_interior_cell)
            .map(|r| r.1)
            .collect()
    }

    pub fn rhs(&self) -> Result<u32> {
        self.0.head.right_leave.context("Missing right leave")
    }
}

pub mod decode {
    use super::*;
    use header::decode::*;
    use nom::multi::fill;
    use nom::{IResult, Parser};

    pub fn parse_offsets(data: &[u8], count: u16) -> IResult<&[u8], Vec<Offset>> {
        let mut vec = vec![Offset::default(); count as usize];
        let res = fill(offset::parser::build, vec.as_mut_slice()).parse(data)?;
        Ok((res.0, vec))
    }

    pub fn take_page(data: &[u8], pad: usize) -> Result<Page> {
        let Ok((head, cells)) = take_header(&data[pad..]).and_then(|(io, head)| {
            let (_, offsets) = parse_offsets(io, head.cell_count)?;
            Ok((head, offsets))
        }) else {
            bail!("Page decoding failed");
        };

        let data = Box::<[u8]>::from(data);
        Ok(Page { head, data, cells })
    }
}
