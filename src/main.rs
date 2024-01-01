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

/// Notes :
/// * Page is loaded into memory in a single read in accordance with SQLite mem approach.
/// * TODO: Improve unit test coverage
/// * TODO: Do not duplicate content for big chunks (str, blob) on cells.

fn main() -> Result<()> {
    // Commands
    let args = args::build()?;

    // Parse command and act accordingly
    let db = SQLiteFile::open_at(&args.filename)?;
    for cmd in args.cmds {
        match cmd {
            Command::Info => {
                println!("database page size: {}", db.head.page_size());
                println!("number of tables: {}", db.schema()?.head.cell_count);
            }
            Command::Tables => {
                let schema = db.schema()?;
                let tables = schema.tables();
                let msg = tables
                    .filter(|c| !c.internal)
                    .map(|t| t.name.as_str())
                    .join(" ");
                println!("{msg}");
            }
            Command::Select(Select::Count { table }) => {
                let page = db.table(&table)?.root;
                println!("{}", page.head.cell_count);
            }
            Command::Select(Select::Column {
                table,
                columns,
                cond,
            }) => {
                let table = db.table(&table)?;
                let res = table.select(&columns, cond);

                for row in res {
                    let line = row.into_iter().map(|v| v.to_string()).join("|");
                    println!("{line}");
                }
            }
        }
    }

    Ok(())
}
