use clap::Args;
use prettytable::{Table, row};

use super::Run;
use crate::{consts, dir::Dir};

#[derive(Args, Debug, PartialEq)]
pub struct Env;

impl Run for Env {
    fn run(&self) -> Result<(), anyhow::Error> {
        let mut table = Table::new();

        table.add_row(row!["Key", "Value", "Explain"]);
        table.add_row(row![
            consts::GOUP_HOME,
            Dir::goup_home().unwrap_or_default().to_string_lossy(),
            "Get goup home directory, default: '$HOME/.goup'",
        ]);
        table.add_row(row![
            consts::GOUP_GO_VERSION,
            consts::go_version().unwrap_or("current".to_owned()),
            "Shell session target go version, default: 'current'",
        ]);
        table.add_row(row![
            consts::GOUP_GO_REGISTRY_INDEX,
            consts::go_registry_index(),
            "Registry index of go version",
        ]);
        table.add_row(row![
            consts::GOUP_GO_REGISTRY,
            consts::go_registry(),
            "Registry of go archive file",
        ]);
        table.add_row(row![
            consts::GOUP_GO_SOURCE_GIT_URL,
            consts::go_source_git_url(),
            "Source git url, use by tip|nightly or index of go version",
        ]);
        table.add_row(row![
            consts::GOUP_GO_SOURCE_GIT_URL,
            consts::go_source_upstream_git_url(),
            "Source upstream git url, use by tip|nightly",
        ]);
        table.printstd();
        Ok(())
    }
}
