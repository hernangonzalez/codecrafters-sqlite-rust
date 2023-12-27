#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Type {
    Null,
    Int(u64),
    Float64,
    Zero,
    One,
    Reserved,
    Blob(u64),
    Text(u64),
}

impl From<u64> for Type {
    fn from(value: u64) -> Self {
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

#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Int(i64),
    Float(f64),
    Blob(Vec<u8>),
    Text(String),
}

impl Value {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Text(s) => Some(&s),
            _ => None,
        }
    }
}

pub mod parser {
    use crate::codec;
    use crate::value::{Type, Value};

    pub fn build(io: &[u8], t: Type) -> anyhow::Result<(&[u8], Value)> {
        anyhow::ensure!(io.len() >= t.len());
        let (buf, io) = io.split_at(t.len());
        let val = match t {
            Type::Null => Value::Null,
            Type::Blob(_) => Value::Blob(buf.to_vec()),
            Type::Text(_) => Value::Text(std::str::from_utf8(buf)?.to_string()),
            Type::Int(_) => Value::Int(codec::two_complements::decode(buf)?),
            Type::Float64 => Value::Float(codec::float::decode(buf)?),
            Type::Reserved => unreachable!("malformed db"),
            Type::Zero => Value::Int(0),
            Type::One => Value::Int(1),
        };
        Ok((io, val))
    }
}
