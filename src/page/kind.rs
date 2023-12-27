use anyhow::anyhow;

#[derive(Debug, Clone, Copy)]
pub enum Kind {
    TableInterior,
    TableLeaf,
    IndexInterior,
    IndexLeaf,
}

impl Kind {
    pub fn is_interior(&self) -> bool {
        match self {
            Self::TableInterior | Self::IndexInterior => true,
            Self::TableLeaf | Self::IndexLeaf => false,
        }
    }
}

impl TryFrom<u8> for Kind {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> anyhow::Result<Self> {
        match value {
            0x5 => Ok(Self::TableInterior),
            0xD => Ok(Self::TableLeaf),
            0x2 => Ok(Self::IndexInterior),
            0xA => Ok(Self::IndexLeaf),
            _ => Err(anyhow!("Unknown page kind")),
        }
    }
}

pub mod parser {
    use super::*;
    use nom::combinator::map_res;
    use nom::number::complete::u8;
    use nom::{IResult, Parser};

    pub fn build(io: &[u8]) -> IResult<&[u8], Kind> {
        map_res(u8, Kind::try_from).parse(io)
    }
}
