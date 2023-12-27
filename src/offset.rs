use std::io::SeekFrom;

#[derive(Debug, Default, Copy, Clone)]
pub struct Offset(u64);

impl Offset {
    pub fn as_usize(&self) -> usize {
        self.0 as usize
    }
}

impl From<u16> for Offset {
    fn from(p: u16) -> Self {
        Self(p as u64)
    }
}

impl From<u64> for Offset {
    fn from(p: u64) -> Self {
        Self(p)
    }
}

impl Into<SeekFrom> for Offset {
    fn into(self) -> SeekFrom {
        SeekFrom::Start(self.0)
    }
}

pub mod parser {
    use super::*;
    use nom::combinator::map;
    use nom::number::complete::be_u16;
    use nom::{IResult, Parser};

    pub fn build(data: &[u8]) -> IResult<&[u8], Offset> {
        map(be_u16, Offset::from).parse(data)
    }
}
