mod install;
mod list;
mod remove;
mod search;
mod set;
mod upgrade;

use std::fs;

use anyhow::anyhow;
use clap::{ArgAction, Args};
use clap::{Parser, Subcommand};
// use derive_more::Display;

use crate::pkg::dir::Dir;

use self::install::Install;
use self::list::List;
use self::remove::Remove;
use self::search::Search;
use self::set::Set;
use self::upgrade::Upgrade;

// run command.
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
    #[command(visible_alias = "ls")]
    List(List),
    /// Remove Go with a version
    #[command(visible_alias = "rm")]
    Remove(Remove),
    /// Search Go versions to install
    Search(Search),
    /// Set the default Go version to one specified.
    /// If no version is provided, a prompt will show to select a installed Go version.
    Set(Set),
    /// Upgrade goup
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

fn switch_version(version: &str) -> Result<(), anyhow::Error> {
    let version = if version.starts_with("go") {
        version.to_string()
    } else {
        format!("go{}", version)
    };
    let home = dirs::home_dir().ok_or_else(|| anyhow!("where is home"))?;

    let version_dir = Dir::new(&home).version(&version);
    if !version_dir.exists() {
        return Err(anyhow!(
            "Go version {version} is not installed. Install it with `goup install`."
        ));
    }
    let current = Dir::new(&home).current();
    fs::remove_file(&current)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs as unix_fs;
        unix_fs::symlink(&version_dir, &current)?;
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs as windows_fs;
        windows_fs::symlink_file(&version_dir, &current)?;
    }
    println!("Default Go is set to '{version}'");
    Ok(())
}
