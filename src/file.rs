use crate::codec;
use anyhow::{Context, Result};
use std::{
    fs::File,
    io::{BufReader, Read},
};

struct Signature {
    format: u32,
}

impl Signature {
    fn read(io: &mut impl Read) -> Result<Self> {
        let mut buf = [0u8; 16];
        io.read_exact(&mut buf)?;
        anyhow::ensure!(buf.starts_with(b"SQLite format "));

        let format = char::from(buf[14])
            .to_digit(10)
            .context("format version #")?;

        Ok(Self { format })
    }
}

struct PageSize(u16);

impl PageSize {
    fn read(io: &mut impl Read) -> Result<Self> {
        codec::num::read_u16(io).map(Self)
    }
}

pub struct SQLiteFile(BufReader<File>);

impl SQLiteFile {
    pub fn open_at(name: &str) -> Result<SQLiteFile> {
        let file = File::open(name)?;
        let mut io = BufReader::new(file);
        let sig = Signature::read(&mut io)?;
        anyhow::ensure!(sig.format == 3);
        Ok(Self(io))
    }

    pub fn page_size(&mut self) -> Result<u16> {
        PageSize::read(&mut self.0).map(|s| s.0)
    }
}
