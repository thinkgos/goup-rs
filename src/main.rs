use clap::Parser;

use goup::command::Cli;
use goup::command::Run;

fn main() -> Result<(), anyhow::Error> {
    Cli::parse().run()
}
