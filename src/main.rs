mod args;
mod codec;
mod file;

use crate::file::SQLiteFile;
use anyhow::Result;
use args::Command;

fn main() -> Result<()> {
    // Commands
    let args = args::build()?;

    // Parse command and act accordingly
    for cmd in args.cmds {
        match cmd {
            Command::Info => {
                let mut file = SQLiteFile::open_at(&args.filename)?;
                let header = file.header()?;
                let page_size = header.page_size();
                println!("database page size: {}", page_size);
            }
        }
    }

    Ok(())
}
