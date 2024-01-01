use crate::value::Value;
use anyhow::{bail, Context, Error, Result};
use itertools::Itertools;
use regex::Regex;
use std::env;

#[derive(Debug)]
pub enum Command {
    Info,
    Tables,
    Select(Select),
}

#[derive(Debug, Clone)]
pub struct ColumnNames(Vec<String>);

impl ColumnNames {
    pub fn as_slice(&self) -> &[String] {
        self.0.as_slice()
    }
}

impl From<&str> for ColumnNames {
    fn from(value: &str) -> Self {
        Self(
            value
                .to_string()
                .split(',')
                .map(|s| s.trim().to_string())
                .collect_vec(),
        )
    }
}

#[derive(Debug)]
pub struct Condition {
    pub name: String,
    pub value: Value,
}

impl TryFrom<&str> for Condition {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self> {
        let mut iter = value.split('=');
        let name = iter.next().context("column")?.trim().to_string();
        let rhs = iter.next().context("value")?.trim();
        let value = rhs.parse::<Value>()?;
        Ok(Self { name, value })
    }
}

#[derive(Debug)]
pub enum Select {
    Count {
        table: String,
    },
    Column {
        table: String,
        columns: ColumnNames,
        cond: Option<Condition>,
    },
}

impl TryFrom<String> for Select {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        let rg_count = Regex::new(r"select count\(\*\) from (?P<table>[A-Za-z]+)")?;
        let rg_col = Regex::new(
            r"(?i)select (?P<columns>[A-Z,\s]+) from (?P<table>[A-Z]+)(\s+where\s+(?P<cond>[A-Z='\s_]+))?",
        )?;
        match value.as_str() {
            s if rg_count.is_match(s) => {
                let caps = rg_count.captures(s).context("select count regex")?;
                let table = (&caps["table"]).to_string();
                Ok(Select::Count { table })
            }
            s if rg_col.is_match(s) => {
                let caps = rg_col.captures(s).context("select col regex")?;
                let table = (&caps["table"]).to_string();
                let columns = (&caps["columns"]).into();
                let cond = caps
                    .name("cond")
                    .map(|m| m.as_str())
                    .map(Condition::try_from)
                    .and_then(|r| r.ok());
                Ok(Select::Column {
                    table,
                    columns,
                    cond,
                })
            }
            e => bail!("Not supported: {e}"),
        }
    }
}

impl TryFrom<String> for Command {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        match value.as_str() {
            ".dbinfo" => Ok(Command::Info),
            ".tables" => Ok(Command::Tables),
            s if s.to_lowercase().starts_with("select") => {
                let sel = Select::try_from(value)?;
                Ok(Command::Select(sel))
            }
            e => bail!("Not a command: {e}"),
        }
    }
}

#[derive(Debug)]
pub struct Args {
    pub filename: String,
    pub cmds: Vec<Command>,
}

pub fn build() -> Result<Args> {
    let mut args = env::args().skip(1);
    let filename = args.next().context("Missing filename")?;
    let cmds = args.map(Command::try_from).collect::<Result<_>>();
    let cmds = cmds?;
    Ok(Args { filename, cmds })
}
