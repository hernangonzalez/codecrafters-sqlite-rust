use anyhow::{bail, Context, Error, Result};
use regex::Regex;
use std::env;

#[derive(Debug)]
pub enum Command {
    Info,
    Tables,
    Select(Select),
}

#[derive(Debug)]
pub enum Select {
    Count { table: String },
    Column { table: String, column: String },
}

impl TryFrom<String> for Select {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        let rg_count = Regex::new(r"select count\(\*\) from (?P<table>[A-Za-z]+)")?;
        let rg_col = Regex::new(r"select (?P<column>[A-Za-z]+) from (?P<table>[A-Za-z]+)")?;
        match value.as_str() {
            s if rg_count.is_match(s) => {
                let caps = rg_count.captures(s).context("select count regex")?;
                let table = (&caps["table"]).to_string();
                Ok(Select::Count { table })
            }
            s if rg_col.is_match(s) => {
                let caps = rg_col.captures(s).context("select col regex")?;
                let table = (&caps["table"]).to_string();
                let column = (&caps["column"]).to_string();
                Ok(Select::Column { table, column })
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
            s if s.starts_with("select") => {
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
