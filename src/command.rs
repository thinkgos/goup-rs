mod cache;
mod completion;
mod env;
#[cfg(unix)]
mod init;
mod install;
mod list;
mod oneself;
mod remove;
mod search;
mod set;
mod shell;

use chrono::Local;
use clap::{ArgAction, Args, CommandFactory};
use clap::{Parser, Subcommand};
use log::LevelFilter;
use shadow_rs::shadow;
use std::env::consts::{ARCH, OS};
use std::io::prelude::Write;

use self::cache::Cache;
use self::completion::Completion;
use self::env::Env;
#[cfg(unix)]
use self::init::Init;
use self::install::Install;
use self::list::List;
use self::oneself::Oneself;
use self::remove::Remove;
use self::search::Search;
use self::set::Set;

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
    #[command(visible_aliases = ["update", "i"])]
    Install(Install),
    /// List all installed Go
    #[command(visible_aliases = ["ls", "show"])]
    List(List),
    /// Remove the specified Go version list.
    /// If no version is provided, a prompt will show to select multiple installed Go version.
    #[command(visible_alias = "rm")]
    Remove(Remove),
    /// Search Go versions to install
    #[command(visible_aliases = ["ls-remote"])]
    Search(Search),
    /// Set the default Go version to one specified.
    /// If no version is provided, a prompt will show to select a installed Go version.
    #[command(name = "default")]
    #[command(visible_aliases = ["set", "use"])]
    Set(Set),
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
    /// Manage cache archive files.
    Cache(Cache),
    /// Using a specific Go version in a shell session.
    /// If no version is provided, a prompt will show to select a installed Go version.
    Shell(shell::Shell),
}

impl Run for Cli {
    fn run(&self) -> Result<(), anyhow::Error> {
        let level_filter = self.global.log_filter_level();
        env_logger::builder()
            .format(move |buf, record| {
                let level = record.level();
                let style = buf.default_level_style(level);
                let time = Local::now().format("%Y-%m-%d %H:%M:%S");
                let args = record.args();
                let target = record.target();
                if level_filter >= LevelFilter::Debug {
                    buf.write_fmt(format_args!(
                        "[{time} {style}{level}{style:#} {target}] {args}\n",
                    ))
                } else {
                    buf.write_fmt(format_args!("[{time} {style}{level}{style:#}] {args}\n",))
                }
            })
            .filter_level(level_filter)
            .init();
        match &self.command {
            Command::Install(cmd) => cmd.run(),
            Command::List(cmd) => cmd.run(),
            Command::Remove(cmd) => cmd.run(),
            Command::Search(cmd) => cmd.run(),
            Command::Set(cmd) => cmd.run(),
            Command::Oneself(cmd) => cmd.run(),
            #[cfg(unix)]
            Command::Init(cmd) => cmd.run(),
            Command::Env(cmd) => cmd.run(),
            Command::Cache(cmd) => cmd.run(),
            Command::Completion(c) => completion::print_completions(c.shell, &mut Self::command()),
            Command::Shell(c) => c.run(),
        }
    }
}
