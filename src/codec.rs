use anyhow::Result;
use std::io::Read;

pub mod num {
    use super::*;

    pub fn read_u16(r: &mut impl Read) -> Result<u16> {
        let mut buf = [0u8; 2];
        r.read_exact(&mut buf)?;
        Ok(u16::from_be_bytes(buf))
    }
}
