use anyhow::{Context, Error, Result};
use std::env;

#[derive(Debug)]
pub enum Command {
    Info,
    Tables,
}

impl TryFrom<String> for Command {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        match value.as_str() {
            ".dbinfo" => Ok(Command::Info),
            ".tables" => Ok(Command::Tables),
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
