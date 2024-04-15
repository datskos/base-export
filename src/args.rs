use crate::Command;
use clap::Parser;

#[derive(Debug, Parser)]
#[clap(name = "app", version)]
pub struct App {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Parser, Debug, Clone)]
#[command(next_help_heading = "block export")]
pub struct BlockArgs {
    /// RPC url
    #[arg(short, long)]
    pub rpc: String,

    /// Output path
    #[arg(short, long)]
    pub path: String,

    /// First block to fetch (inclusive)
    #[arg(long, default_value_t = 1)]
    pub start: u64,

    /// Last block to fetch (inclusive)
    #[arg(long)]
    pub end: u64,
}
