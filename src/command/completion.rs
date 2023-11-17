use std::io;

use clap::{Args, Command};
use clap_complete::{generate, Generator, Shell};

#[derive(Args, Debug, PartialEq)]
pub struct Completion {
    // If provided, outputs the completion file for given shell
    #[arg(value_enum)]
    pub shell: Shell,
}

pub fn print_completions<G: Generator>(gen: G, cmd: &mut Command) -> Result<(), anyhow::Error> {
    generate(gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
    Ok(())
}
