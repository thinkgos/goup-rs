use clap::Args;
use goup_version::consts;
use prettytable::{Table, row};

use super::Run;

#[derive(Args, Debug, PartialEq)]
pub struct Env;

impl Run for Env {
    fn run(&self) -> Result<(), anyhow::Error> {
        let mut table = Table::new();

        table.add_row(row!["Key", "Value", "Explain"]);
        table.add_row(row![
            consts::GOUP_GO_HOST,
            consts::go_host(),
            "Get upstream latest go version, use by 'install'/'search'",
        ]);
        table.add_row(row![
            consts::GOUP_GO_DOWNLOAD_BASE_URL,
            consts::go_download_base_url(),
            "Download go archive file base url, use by 'install'",
        ]);
        table.add_row(row![
            consts::GOUP_GO_SOURCE_GIT_URL,
            consts::go_source_git_url(),
            "Upstream source git url and get upstream go versions, use by 'install'/'search'",
        ]);
        table.add_row(row![
            consts::GOUP_GO_SOURCE_GIT_URL,
            consts::go_source_upstream_git_url(),
            "Upstream source git url, use by 'install' the gotip",
        ]);
        table.printstd();
        Ok(())
    }
}
