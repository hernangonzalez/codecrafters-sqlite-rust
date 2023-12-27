use crate::Result;

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CodecError {
    /// Not enough input bytes.
    Insufficient,
    /// Input bytes exceed maximum.
    Overflow,
}

impl std::fmt::Display for CodecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Coding error: {}", *self as u8)
    }
}

pub mod varint {
    use super::*;
    use itertools::{FoldWhile, Itertools};
    const MSB: u8 = 0b1000_0000;

    #[inline]
    fn is_last(b: u8) -> bool {
        b & MSB == 0
    }

    #[inline]
    fn drop_msb(b: u8) -> u8 {
        b & !MSB
    }

    #[inline]
    pub fn decode(src: &[u8]) -> Result<(&[u8], u64)> {
        anyhow::ensure!(src.len() > 0, CodecError::Insufficient);

        let iter = src.iter();
        let res = iter
            .copied()
            .enumerate()
            .fold_while((0, 0), |(_, acc), (i, x)| {
                let mut acc = acc << 7;
                acc |= u64::from(drop_msb(x));
                if is_last(x) {
                    FoldWhile::Done((i, acc))
                } else {
                    FoldWhile::Continue((i, acc))
                }
            });

        match res {
            FoldWhile::Done((i, val)) => Ok((&src[i + 1..], val)),
            FoldWhile::Continue(_) => anyhow::bail!(CodecError::Overflow),
        }
    }
}

pub mod two_complements {
    use super::*;

    pub fn decode(src: &[u8]) -> Result<i64> {
        anyhow::ensure!(src.len() <= 8);
        let mut buf = [0u8; 8];
        let start = buf.len() - src.len();
        for (i, b) in src.iter().enumerate() {
            buf[start + i] = !b
        }
        Ok(i64::from_be_bytes(buf) + 0x1)
    }
}

pub mod float {
    use super::*;

    pub fn decode(src: &[u8]) -> Result<f64> {
        anyhow::ensure!(src.len() == 8);
        let mut buf = [0u8; 8];
        buf.copy_from_slice(src);
        Ok(f64::from_be_bytes(buf))
    }
}
