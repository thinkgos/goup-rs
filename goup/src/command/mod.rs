mod completion;
mod downloads;
mod env;
#[cfg(unix)]
mod init;
mod install;
mod list;
mod oneself;
mod remove;
mod search;
mod set;
mod upgrade;

use clap::{ArgAction, Args, CommandFactory};
use clap::{Parser, Subcommand};
use log::LevelFilter;
use shadow_rs::shadow;
use std::env::consts::{ARCH, OS};

use self::completion::Completion;
use self::downloads::Downloads;
use self::env::Env;
#[cfg(unix)]
use self::init::Init;
use self::install::Install;
use self::list::List;
use self::oneself::Oneself;
use self::remove::Remove;
use self::search::Search;
use self::set::Set;
use self::upgrade::Upgrade;

shadow!(build);
const VERSION: &str = shadow_rs::formatcp!(
    r#"{} 
-------------------------------------
{}

Author:          {}
Email:           {}
Repository:      {}
Branch:          {}
GitCommit:       {}
GitFullCommit:   {}
BuildTime:       {}
BuildEnv:        {}, {}
BuildOs:         {}
BuildArch:       {}"#,
    env!("CARGO_PKG_VERSION"),
    env!("CARGO_PKG_DESCRIPTION"),
    build::COMMIT_AUTHOR,
    build::COMMIT_EMAIL,
    env!("CARGO_PKG_REPOSITORY"),
    build::BRANCH,
    build::SHORT_COMMIT,
    build::COMMIT_HASH,
    build::BUILD_TIME_2822,
    build::RUST_VERSION,
    build::RUST_CHANNEL,
    OS,
    ARCH,
);

// run command.
pub trait Run {
    fn run(&self) -> Result<(), anyhow::Error>;
}

#[derive(Args, Debug, PartialEq)]
struct Global {
    /// Verbose log
    #[arg(short, long, action = ArgAction::Count)]
    verbose: u8,
    /// Whether or not to write the target in the log format.
    #[arg(short, long)]
    enable_target: bool,
}

impl Global {
    fn log_filter_level(&self) -> LevelFilter {
        match self.verbose {
            0 => LevelFilter::Info,
            1 => LevelFilter::Debug,
            _ => LevelFilter::Trace,
        }
    }
}

#[derive(Parser, Debug, PartialEq)]
#[command(author, about, long_about = None)]
#[command(propagate_version = true)]
#[command(version = VERSION)]
#[command(name = "goup")]
pub struct Cli {
    #[command(flatten)]
    global: Global,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug, PartialEq)]
#[non_exhaustive] // 表明未来还有其它元素添加
enum Command {
    /// Install Go with a version
    #[command(visible_alias = "update")]
    Install(Install),
    /// List all installed Go
    #[command(visible_aliases = ["ls", "show"])]
    List(List),
    /// Remove the specified Go version list.
    /// If no version is provided, a prompt will show to select multiple installed Go version.
    #[command(visible_alias = "rm")]
    Remove(Remove),
    /// Search Go versions to install
    Search(Search),
    /// Set the default Go version to one specified.
    /// If no version is provided, a prompt will show to select a installed Go version.
    #[command(visible_alias = "use")]
    Set(Set),
    /// Upgrade goup, deprecated in future version, use `goup self update` instead
    Upgrade(Upgrade),
    /// Generate the autocompletion script for the specified shell
    Completion(Completion),
    #[cfg(unix)]
    /// write all necessary environment variables and values.
    Init(Init),
    /// Show the specified goup environment variables and values.
    Env(Env),
    /// Modify the goup installation.
    #[command(name = "self")]
    Oneself(Oneself),
    /// Manage download archive files.
    #[command(visible_alias = "dl")]
    Downloads(Downloads),
}

impl Run for Cli {
    fn run(&self) -> Result<(), anyhow::Error> {
        env_logger::builder()
            .format_target(self.global.enable_target)
            .filter_level(self.global.log_filter_level())
            .init();
        match &self.command {
            Command::Install(cmd) => cmd.run(),
            Command::List(cmd) => cmd.run(),
            Command::Remove(cmd) => cmd.run(),
            Command::Search(cmd) => cmd.run(),
            Command::Set(cmd) => cmd.run(),
            Command::Upgrade(cmd) => cmd.run(),
            Command::Oneself(cmd) => cmd.run(),
            #[cfg(unix)]
            Command::Init(cmd) => cmd.run(),
            Command::Env(cmd) => cmd.run(),
            Command::Downloads(cmd) => cmd.run(),
            Command::Completion(c) => completion::print_completions(c.shell, &mut Self::command()),
        }
    }
}
