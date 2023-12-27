mod args;
mod codec;
mod file;
mod offset;
mod page;
mod value;

use crate::page::Cell;
use anyhow::Result;
use args::Command;
use file::SQLiteFile;
use itertools::Itertools;

fn main() -> Result<()> {
    // Commands
    let args = args::build()?;

    // Parse command and act accordingly
    let mut file = SQLiteFile::open_at(&args.filename)?;
    for cmd in args.cmds {
        match cmd {
            Command::Info => {
                println!("database page size: {}", file.head.page_size());
                println!("number of tables: {}", file.schema()?.head.cell_count);
            }
            Command::Tables => {
                let tables = file.tables()?;
                let msg = tables
                    .iter()
                    .filter(|c| !c.is_internal())
                    .flat_map(|t| t.record.values[2].as_str())
                    .join(" ");
                println!("{msg}");
            }
        }
    }

    Ok(())
}
