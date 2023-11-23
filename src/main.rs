use clap::Parser;

use goup::Cli;
use goup::Run;

fn main() -> Result<(), anyhow::Error> {
    Cli::parse().run()
}
