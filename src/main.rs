use clap::Parser;

use goup_rs::Cli;
use goup_rs::Run;

fn main() -> Result<(), anyhow::Error> {
    Cli::parse().run()
}
