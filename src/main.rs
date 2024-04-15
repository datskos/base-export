use crate::args::{App, BlockArgs};
use crate::export::BlockWriter;
use clap::{Parser, Subcommand};
use std::ops::RangeInclusive;

mod args;
mod codec;
mod export;

#[derive(Debug, Subcommand)]
enum Command {
    Blocks(BlockArgs),
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let app = App::parse();
    match app.command {
        Command::Blocks(args) => {
            let client = BlockWriter::new(args.rpc)?;
            client
                .write(RangeInclusive::new(args.start, args.end), args.path)
                .await?;
        }
    }

    Ok(())
}
