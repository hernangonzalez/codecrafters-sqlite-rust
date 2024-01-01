use crate::Result;
use nom::bytes::complete::{tag, take_until, take_while};
use nom::character::complete::{char, multispace0};
use nom::multi::separated_list0;
use nom::sequence::preceded;
use nom::{IResult, Parser};

pub mod varint {
    use super::*;

    pub fn take(io: &[u8]) -> IResult<&[u8], i64> {
        let mut varint: i64 = 0;
        let mut bytes_read: usize = 0;
        for (i, byte) in io.iter().enumerate().take(9) {
            bytes_read += 1;
            if i == 8 {
                varint = (varint << 8) | *byte as i64;
                break;
            } else {
                varint = (varint << 7) | (*byte & 0b0111_1111) as i64;
                if *byte < 0b1000_0000 {
                    break;
                }
            }
        }

        Ok((&io[bytes_read..], varint))
    }
}

pub mod two_complements {
    use super::*;

    pub fn decode(src: &[u8]) -> Result<i64> {
        anyhow::ensure!(src.len() <= 8);
        let val = match src.len() {
            1 => u8::from_be_bytes([src[0]]) as i64,
            len => {
                let mut buf = [0u8; 8];
                let start = buf.len() - len;
                for (i, b) in src.iter().enumerate() {
                    buf[start + i] = !b
                }
                i64::from_be_bytes(buf) + 0x1
            }
        };
        Ok(val)
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

pub mod sql {
    use super::*;

    fn extract_name(io: &str) -> IResult<&str, &str> {
        take_while(|c| c != ',')
            .and_then(preceded(
                multispace0,
                take_while(|c: char| c.is_ascii_graphic()),
            ))
            .parse(io)
    }

    fn parse_between(io: &str) -> IResult<&str, Vec<&str>> {
        let mut drop_statement = take_until("(");
        let (io, _) = drop_statement.parse(io)?;

        preceded(tag("("), take_until(")"))
            .and_then(separated_list0(char(','), extract_name))
            .parse(io)
    }

    pub fn column_names(io: &str) -> Vec<&str> {
        parse_between(io).map(|r| r.1).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const SQL_CREATE: &str = "CREATE TABLE butterscotch (id integer primary key, grape text,eye_color text,coconut text,coffee text,butterscotch text)";

    #[test]
    fn test_sql_column_names() {
        let names = sql::column_names(SQL_CREATE);
        assert_eq!(
            &names,
            &[
                "id",
                "grape",
                "eye_color",
                "coconut",
                "coffee",
                "butterscotch"
            ]
        );
    }
}
