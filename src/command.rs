mod cache;
mod completion;
mod default;
mod env;
mod init;
mod install;
mod list;
mod oneself;
mod remove;
mod search;
mod shell;
mod utils;

use clap::CommandFactory;
use clap::{Parser, Subcommand};
use env_logger::Env as LoggerEnv;
use shadow_rs::shadow;
use std::env::consts::{ARCH, OS};
use std::io::prelude::Write;

use self::cache::Cache;
use self::completion::Completion;
use self::default::Default;
use self::env::Env;
use self::init::Init;
use self::install::Install;
use self::list::List;
use self::oneself::Oneself;
use self::remove::Remove;
use self::search::Search;

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
pub(crate) trait Run {
    fn run(&self) -> Result<(), anyhow::Error>;
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
    /// Set the default Go version.
    /// If no version is provided, a prompt will show to select a installed Go version.
    #[command(visible_aliases = ["use", "set"])]
    Default(Default),
    /// Generate the autocompletion script for the specified shell
    Completion(Completion),
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

impl Run for Command {
    fn run(&self) -> Result<(), anyhow::Error> {
        match self {
            Command::Install(cmd) => cmd.run(),
            Command::List(cmd) => cmd.run(),
            Command::Remove(cmd) => cmd.run(),
            Command::Search(cmd) => cmd.run(),
            Command::Default(cmd) => cmd.run(),
            Command::Oneself(cmd) => cmd.run(),
            Command::Init(cmd) => cmd.run(),
            Command::Env(cmd) => cmd.run(),
            Command::Cache(cmd) => cmd.run(),
            Command::Completion(c) => completion::print_completions(c.shell, &mut Cli::command()),
            Command::Shell(c) => c.run(),
        }
    }
}

#[derive(Parser, Debug, PartialEq)]
#[command(author, about, long_about = None)]
#[command(propagate_version = true)]
#[command(version = VERSION)]
#[command(name = "goup")]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

impl Cli {
    fn run_command(&self) -> Result<(), anyhow::Error> {
        self.command.run()
    }
    pub fn run_main() -> Result<(), anyhow::Error> {
        env_logger::builder()
            .format(move |buf, record| {
                let level = record.level();
                let style = buf.default_level_style(level);
                let time = jiff::Zoned::now().strftime("%F %T%.8f %z %Z");
                let args = record.args();
                let target = record.target();
                buf.write_fmt(format_args!(
                    "[{time} {style}{level}{style:#} {target}] {args}\n"
                ))
            })
            .parse_env(LoggerEnv::default().default_filter_or("info"))
            .init();
        Self::parse().run_command()
    }
}
