use clap::Parser;

use govm::command::Cli;
use govm::command::Run;

fn main() -> Result<(), anyhow::Error> {
    Cli::parse().run()
}
