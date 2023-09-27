use clap::Parser;

use godl::command::Cli;
use godl::command::Run;

fn main() -> Result<(), anyhow::Error> {
    Cli::parse().run()
}
