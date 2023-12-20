mod args;

use anyhow::Result;
use args::Command;
use std::fs::File;
use std::io::prelude::*;

fn main() -> Result<()> {
    // Commands
    let args = args::build()?;

    // Parse command and act accordingly
    for cmd in args.cmds {
        match cmd {
            Command::Info => {
                let mut file = File::open(&args.filename)?;
                let mut header = [0; 100];
                file.read_exact(&mut header)?;

                let page_size = u16::from_be_bytes([header[16], header[17]]);

                println!("database page size: {}", page_size);
            }
        }
    }

    Ok(())
}
