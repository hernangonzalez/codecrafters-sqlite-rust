use crate::codec;
use anyhow::{bail, Error, Result};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Type {
    Null,
    Int(i64),
    Float64,
    Zero,
    One,
    Reserved,
    Blob(i64),
    Text(i64),
}

impl From<i64> for Type {
    fn from(value: i64) -> Self {
        match value {
            0 => Self::Null,
            n if (1..4).contains(&n) => Self::Int(n),
            5 => Self::Int(6),
            6 => Self::Int(8),
            7 => Self::Float64,
            8 => Self::Zero,
            9 => Self::One,
            10 | 11 => Self::Reserved,
            n if (n % 2 == 0) => Self::Blob((n - 12) / 2),
            n if (n % 2 != 0) => Self::Text((n - 13) / 2),
            _ => unreachable!(),
        }
    }
}

impl Type {
    fn len(&self) -> usize {
        match self {
            Self::Null => 0,
            Self::Int(l) => *l as usize,
            Self::Float64 => 8,
            Self::Zero => 0,
            Self::One => 0,
            Self::Reserved => unreachable!("DB is not well-formed"),
            Self::Blob(l) => *l as usize,
            Self::Text(l) => *l as usize,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Int(i64),
    Float(f64),
    Blob(Vec<u8>),
    Text(String),
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Text(s) => s.fmt(f),
            Value::Float(n) => n.fmt(f),
            Value::Int(i) => i.fmt(f),
            Value::Blob(_) => todo!(),
        }
    }
}

impl TryInto<i64> for &Value {
    type Error = Error;

    fn try_into(self) -> Result<i64> {
        match self {
            Value::Null => bail!("null"),
            Value::Float(n) => Ok(*n as i64),
            Value::Int(i) => Ok(*i),
            _ => bail!("NaN"),
        }
    }
}

impl FromStr for Value {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "NULL" => Ok(Value::Null),
            s if s.starts_with('\'') && s.ends_with('\'') => {
                Ok(Value::Text(s[1..s.len() - 1].to_string()))
            }
            s if s.chars().all(|c| c.is_digit(10)) => Ok(Value::Int(s.parse()?)),
            _ => bail!("Unsupported format"),
        }
    }
}

impl Value {
    pub fn decode(chunk: &[u8], t: Type) -> Result<Self> {
        let val = match t {
            Type::Null => Value::Null,
            Type::Blob(_) => Value::Blob(chunk.to_vec()),
            Type::Text(_) => Value::Text(std::str::from_utf8(chunk)?.to_string()),
            Type::Int(_) => Value::Int(codec::two_complements::decode(chunk)?),
            Type::Float64 => Value::Float(codec::float::decode(chunk)?),
            Type::Reserved => bail!("malformed db"),
            Type::Zero => Value::Int(0),
            Type::One => Value::Int(1),
        };
        Ok(val)
    }
}

pub mod decode {
    use crate::codec::varint;
    use crate::value::{Type, Value};
    use nom::bytes::complete::take;
    use nom::combinator::map_res;
    use nom::{IResult, Parser};

    pub fn take_value(io: &[u8], t: Type) -> IResult<&[u8], Value> {
        map_res(take(t.len()), |c| Value::decode(c, t)).parse(io)
    }

    pub fn take_type(io: &[u8]) -> IResult<&[u8], Type> {
        varint::take.map(Type::from).parse(io)
    }
}
