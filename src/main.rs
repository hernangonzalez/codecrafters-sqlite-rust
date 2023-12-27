mod args;
mod codec;
mod file;
mod offset;
mod page;
mod schema;
mod value;

use anyhow::{Context, Result};
use args::{Command, Select};
use file::{SQLiteFile, SQL};
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
                let schema = file.schema()?;
                let table = schema.table_named(&table)?;
                let page = file.page_at(table.id)?;
                println!("{}", page.head.cell_count);
            }
            Command::Select(Select::Column { table, column }) => {
                let schema = file.schema()?;
                let table = schema.table_named(&table)?;
                let page = file.page_at(table.id)?;
                let cols = table.column_names();
                let col_idx = cols
                    .iter()
                    .find_position(|c| *c == &column.as_str())
                    .context("column not found")?
                    .0;

                let cells = page.cells();
                let names = cells
                    .iter()
                    .flat_map(|cell| cell.record.values.get(col_idx))
                    .map(|v| v.to_string());

                names.for_each(|name| println!("{name}"));
            }
        }
    }

    Ok(())
}
