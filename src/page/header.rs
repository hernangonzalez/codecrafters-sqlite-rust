use super::kind::{self, Kind};

#[derive(Debug)]
pub struct Header {
    pub kind: Kind,
    pub free_block: u16,
    pub cell_count: u16,
    cell_content: u16,
    pub fragment_count: u8,
    pub right_leave: Option<u32>,
}

#[allow(dead_code)]
impl Header {
    fn cell_content(&self) -> u32 {
        match self.cell_content {
            0 => 65_536,
            _ => self.cell_content as u32,
        }
    }
}

pub mod parser {
    use super::*;
    use nom::combinator::{cond, map};
    use nom::number::complete::{be_u16, be_u32, u8};
    use nom::sequence::tuple;
    use nom::{IResult, Parser};

    pub fn build(io: &[u8]) -> IResult<&[u8], Header> {
        let (io, kind) = kind::parser::build(io)?;
        let fields = (be_u16, be_u16, be_u16, u8, cond(kind.is_interior(), be_u32));
        map(tuple(fields), move |t| Header {
            kind,
            free_block: t.0,
            cell_count: t.1,
            cell_content: t.2,
            fragment_count: t.3,
            right_leave: t.4,
        })
        .parse(io)
    }
}
