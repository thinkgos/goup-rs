use std::io;

use clap::{Args, Command};
use clap_complete::{Generator, Shell};

#[derive(Args, Debug, PartialEq)]
pub struct Completion {
    // outputs the completion content for given shell
    #[arg(value_enum)]
    pub shell: Shell,
}

pub fn print_completions<G: Generator>(r#gen: G, cmd: &mut Command) -> Result<(), anyhow::Error> {
    clap_complete::generate(r#gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
    Ok(())
}
