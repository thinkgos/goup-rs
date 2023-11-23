mod completion;
#[cfg(unix)]
mod init;
mod install;
mod list;
mod remove;
mod search;
mod set;
mod upgrade;

use clap::{ArgAction, Args, CommandFactory};
use clap::{Parser, Subcommand};
use shadow_rs::shadow;
use std::env::consts::{ARCH, OS};

use self::completion::Completion;
#[cfg(unix)]
use self::init::Init;
use self::install::Install;
use self::list::List;
use self::remove::Remove;
use self::search::Search;
use self::set::Set;
use self::upgrade::Upgrade;

shadow!(build);
const VERSION: &str = shadow_rs::formatcp!(
    r#"{}
auth:            {}
git_commit:      {}
git_full_commit: {}
build_time:      {}
build_env:       {},{}
build_os:        {}
build_arch:      {}"#,
    build::PKG_VERSION,
    build::COMMIT_AUTHOR,
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
    /// Verbose
    #[arg(short, long, action = ArgAction::Count)]
    verbose: u8,
}

#[derive(Parser, Debug, PartialEq)]
#[command(author, about, long_about = None)]
#[command(propagate_version = true)]
#[command(version = VERSION)]
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
    /// Remove Go with multiple version.
    /// If no version is provided, a prompt will show to select multiple installed Go version.
    #[command(visible_alias = "rm")]
    Remove(Remove),
    /// Search Go versions to install
    Search(Search),
    /// Set the default Go version to one specified.
    /// If no version is provided, a prompt will show to select a installed Go version.
    #[command(visible_alias = "use")]
    Set(Set),
    /// Upgrade goup
    Upgrade(Upgrade),
    /// Generate the autocompletion script for the specified shell
    Completion(Completion),
    #[cfg(unix)]
    /// write all necessary environment variables and values.
    Init(Init),
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
            #[cfg(unix)]
            Command::Init(cmd) => cmd.run(),
            Command::Completion(c) => completion::print_completions(c.shell, &mut Self::command()),
        }
    }
}
