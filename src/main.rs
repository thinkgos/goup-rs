use clap::Parser;

use goup_rs::command::Cli;
use goup_rs::command::Run;

fn main() -> Result<(), anyhow::Error> {
    Cli::parse().run()
}
