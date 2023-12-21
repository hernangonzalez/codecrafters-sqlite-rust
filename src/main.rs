mod args;
mod codec;
mod file;

use anyhow::Result;
use args::Command;
use file::SQLiteFile;

fn main() -> Result<()> {
    // Commands
    let args = args::build()?;

    // Parse command and act accordingly
    for cmd in args.cmds {
        match cmd {
            Command::Info => {
                let mut file = SQLiteFile::open_at(&args.filename)?;
                println!("database page size: {}", file.head.page_size());
                println!("number of tables: {}", file.schema()?.row_count());
            }
        }
    }

    Ok(())
}
