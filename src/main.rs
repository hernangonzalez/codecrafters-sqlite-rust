mod args;
mod codec;
mod db;
mod offset;
mod page;
mod schema;
mod value;

use anyhow::Result;
use args::{Command, Select};
use db::{SQLiteFile, SQL};
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
                let schema = file.schema()?;
                let tables = schema.tables();
                let msg = tables
                    .filter(|c| !c.internal)
                    .map(|t| t.name.as_str())
                    .join(" ");
                println!("{msg}");
            }
            Command::Select(Select::Count { table }) => {
                let page = file.select(table)?.root;
                println!("{}", page.head.cell_count);
            }
            Command::Select(Select::Column {
                table,
                columns,
                cond,
            }) => {
                let table = file.select(table)?;
                let cols = table.select(&columns, cond);

                cols.iter().for_each(|row| {
                    let line = row.iter().map(|v| v.to_string()).join("|");
                    println!("{line}");
                });
            }
        }
    }

    Ok(())
}
