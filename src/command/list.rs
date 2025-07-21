use clap::Args;
use prettytable::{Table, row};
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
            let mut table = Table::new();
            table.add_row(row!["Active", "Version"]);
            for v in vers {
                if v.active {
                    table.add_row(row![Fycb -> "*", Fycb -> &v.version]);
                } else {
                    table.add_row(row![Fgc -> "", Fgc -> &v.version]);
                };
            }
            table.printstd();
        }
        Ok(())
    }
}
