use anyhow::{Context, Error, Result};
use regex::Regex;
use std::env;

#[derive(Debug)]
pub enum Command {
    Info,
    Tables,
    Count(String),
}

impl TryFrom<String> for Command {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        let rg_select = Regex::new(r"select count\(\*\) from (?P<table_name>[A-Za-z]+)")?;

        match value.as_str() {
            ".dbinfo" => Ok(Command::Info),
            ".tables" => Ok(Command::Tables),
            count if (rg_select.is_match(count)) => {
                let caps = rg_select.captures(count).context("regex")?;
                let name = (&caps["table_name"]).to_string();
                Ok(Command::Count(name))
            }
            e => Err(anyhow::anyhow!("Not a command: {e}")),
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
    let cmds = args.flat_map(Command::try_from).collect();
    Ok(Args { filename, cmds })
}
