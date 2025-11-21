use std::io::{self, Write};

use clap::Args;

use super::Run;
use crate::{consts, dir::Dir};

#[derive(Args, Debug, PartialEq)]
pub struct Env;

impl Run for Env {
    fn run(&self) -> Result<(), anyhow::Error> {
        let envs = [
            ("Key", "Value".to_owned(), "Explain"),
            (
                consts::GOUP_HOME,
                Dir::goup_home()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                "Get goup home directory, default: '$HOME/.goup'",
            ),
            (
                consts::GOUP_GO_VERSION,
                consts::go_version().unwrap_or("current".to_owned()),
                "Shell session target go version, default: 'current'",
            ),
            (
                consts::GOUP_GO_REGISTRY_INDEX,
                consts::go_registry_index(),
                "Registry index of go version",
            ),
            (
                consts::GOUP_GO_REGISTRY,
                consts::go_registry(),
                "Registry of go archive file",
            ),
            (
                consts::GOUP_GO_SOURCE_GIT_URL,
                consts::go_source_git_url(),
                "Source git url, use by tip|nightly or index of go version",
            ),
            (
                consts::GOUP_GO_SOURCE_GIT_URL,
                consts::go_source_upstream_git_url(),
                "Source upstream git url, use by tip|nightly",
            ),
        ];

        let mut stdout = io::stdout().lock();
        let write_separator = |write: &mut dyn Write| {
            writeln!(
                write,
                "+-{}-+-{}-+-{}-+",
                "-".repeat(22),
                "-".repeat(30),
                "-".repeat(60)
            )
        };
        write_separator(&mut stdout)?;
        for (i, (k, v, e)) in envs.iter().enumerate() {
            writeln!(stdout, "| {:<22} | {:<30} | {:<60} |", k, v, e)?;
            if i == 0 {
                write_separator(&mut stdout)?;
            }
        }
        write_separator(&mut stdout)?;
        stdout.flush()?;
        Ok(())
    }
}
