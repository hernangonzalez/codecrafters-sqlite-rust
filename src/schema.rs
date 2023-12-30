use crate::codec;
use crate::page::{Header, Page, TableLeafCell};
use anyhow::{bail, Context, Error, Result};
use itertools::Itertools;

const NAME_PREFIX_SQLITE: &str = "sqlite_";

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Type {
    Table,
    Index,
    View,
    Trigger,
}

impl TryFrom<&str> for Type {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self> {
        match value {
            "table" => Ok(Self::Table),
            "index" => Ok(Self::Index),
            "view" => Ok(Self::View),
            "trigger" => Ok(Self::Trigger),
            e => bail!("Unknown type: {e}"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Descriptor {
    pub id: u64,
    pub name: String,
    pub kind: Type,
    pub internal: bool,
    pub sql: String,
}

impl TryFrom<&TableLeafCell> for Descriptor {
    type Error = Error;

    fn try_from(r: &TableLeafCell) -> Result<Self> {
        let id = r.id;
        let name = parser::name(r)?;
        let kind = parser::kind(r)?;
        let internal = name.starts_with(NAME_PREFIX_SQLITE);
        let sql = parser::sql(r)?;
        Ok(Self {
            id,
            name,
            kind,
            internal,
            sql,
        })
    }
}

impl Descriptor {
    pub fn column_names(&self) -> Vec<&str> {
        codec::sql::column_names(&self.sql)
    }
}

pub struct Schema {
    pub head: Header,
    pub desc: Vec<Descriptor>,
}

impl TryFrom<Page> for Schema {
    type Error = Error;

    fn try_from(page: Page) -> Result<Self> {
        let cells = page.cells();
        let head = page.head;
        let desc = cells
            .iter()
            .map(|c| Descriptor::try_from(c))
            .filter_ok(|t| t.kind == Type::Table)
            .try_collect()?;
        Ok(Self { head, desc })
    }
}

impl Schema {
    pub fn tables(&self) -> impl Iterator<Item = &Descriptor> {
        self.desc.iter().filter(|t| t.kind == Type::Table)
    }

    pub fn table_named(&self, name: &str) -> Result<&Descriptor> {
        self.desc
            .iter()
            .find(|d| d.name == name)
            .context("table not found")
    }
}

mod parser {
    use super::*;
    use crate::value::Value;

    pub fn kind(c: &TableLeafCell) -> Result<Type> {
        let value = c.record.values.get(0).context("type")?;
        let Value::Text(s) = value else {
            bail!("invalid type")
        };
        s.as_str().try_into()
    }

    pub fn name(c: &TableLeafCell) -> Result<String> {
        let value = c.record.values.get(1).context("name")?;
        Ok(value.to_string())
    }

    pub fn sql(c: &TableLeafCell) -> Result<String> {
        let value = c.record.values.get(4).context("sql statement")?;
        Ok(value.to_string())
    }
}
