use std::io::{self, Write};

use clap::Args;
use owo_colors::OwoColorize;
use which::which;

use super::Run;
use crate::version::Version;

#[derive(Args, Debug, PartialEq)]
pub struct List;

impl Run for List {
    fn run(&self) -> Result<(), anyhow::Error> {
        let vers = Version::list_go_version()?;
        if vers.is_empty() {
            log::info!(
                "No Go is installed by goup.{}",
                if let Ok(go_bin) = which("go") {
                    format!(" Using system Go {}.", go_bin.to_string_lossy())
                } else {
                    "".to_owned()
                }
            );
        } else {
            let mut stdout = io::stdout().lock();
            for v in vers {
                match (v.default, v.session) {
                    (true, true) => writeln!(
                        stdout,
                        "{:<10}{}",
                        v.version.yellow(),
                        "(active, default & session)".yellow()
                    )?,
                    (true, _) => writeln!(
                        stdout,
                        "{:<10}{}",
                        v.version.yellow(),
                        "(active, default)".yellow()
                    )?,
                    (_, true) => writeln!(
                        stdout,
                        "{:<10}{}",
                        v.version.yellow().dimmed(),
                        "(active, session)".yellow().dimmed()
                    )?,
                    _ => writeln!(stdout, "{:<10}", v.version)?,
                };
            }
            stdout.flush()?;
        }
        Ok(())
    }
}
