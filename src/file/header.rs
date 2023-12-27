use std::io::Read;

const HEADER_SIZE: usize = 100;

pub struct Header([u8; HEADER_SIZE]);

#[allow(dead_code)]
impl Header {
    pub const fn size() -> usize {
        HEADER_SIZE
    }

    pub fn read(io: &mut impl Read) -> anyhow::Result<Self> {
        let mut buf = [0u8; 100];
        io.read_exact(&mut buf)?;
        anyhow::ensure!(buf.starts_with(b"SQLite format 3\0"));
        Ok(Self(buf))
    }

    pub fn page_size(&self) -> u32 {
        let size = u16::from_be_bytes([self.0[16], self.0[17]]);
        match size {
            1 => 65_536u32,
            s => s as u32,
        }
    }

    /// Reserved chunk at the end of each page.
    pub fn reserved_page_size(&self) -> u8 {
        self.0[20]
    }
}
