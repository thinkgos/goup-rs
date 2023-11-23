use clap::Parser;

use govm::Cli;
use govm::Run;

fn main() -> Result<(), anyhow::Error> {
    Cli::parse().run()
}
