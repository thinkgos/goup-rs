use clap::Args;
use colored::Colorize;
use prettytable::{row, Table};
use which::which;

use super::Run;
use goup_version::Version;

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
            let star = "*".yellow().bold().to_string();
            let mut table = Table::new();

            table.add_row(row!["Active", "Version"]);
            for v in vers {
                let (active, version) = if v.active {
                    (star.as_ref(), v.version.yellow().bold())
                } else {
                    ("", v.version.green())
                };
                table.add_row(row![bc -> active, version]);
            }
            table.printstd();
        }
        Ok(())
    }
}
