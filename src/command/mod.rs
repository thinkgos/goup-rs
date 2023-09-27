mod install;
mod list;
mod remove;
mod search;
mod set;
mod upgrade;

// use std::path::PathBuf;

use anyhow::anyhow;
use clap::{ArgAction, Args};
use clap::{Parser, Subcommand};
// use derive_more::Display;

use self::install::Install;
use self::list::List;
use self::remove::Remove;
use self::search::Search;
use self::set::Set;
use self::upgrade::Upgrade;

pub trait Run {
    fn run(&self) -> Result<(), anyhow::Error>;
}

#[derive(Args, Debug)]
pub struct Global {
    /// Verbose
    #[arg(short, long, action = ArgAction::Count)]
    verbose: u8,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(flatten)]
    pub global: Global,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
#[non_exhaustive] // 表明未来还有其它元素添加
pub enum Command {
    /// Install Go with a version
    Install(Install),
    /// List all installed Go
    #[command(alias = "ls")]
    List(List),
    /// Remove Go with a version
    Remove(Remove),
    /// Search Go versions to install
    Search(Search),
    /// Set the default Go version
    Set(Set),
    /// Upgrade godl
    Upgrade(Upgrade),
}

impl Run for Cli {
    fn run(&self) -> Result<(), anyhow::Error> {
        match &self.command {
            Command::Install(cmd) => cmd.run(),
            Command::List(cmd) => cmd.run(),
            Command::Remove(cmd) => cmd.run(),
            Command::Search(cmd) => cmd.run(),
            Command::Set(cmd) => cmd.run(),
            Command::Upgrade(cmd) => cmd.run(),
        }
    }
}
