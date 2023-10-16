use clap::Args;
use prettytable::{row, Table};
use which::which;

use super::Run;
use crate::pkg::version::Version;

#[derive(Args, Debug)]
pub struct List;

impl Run for List {
    fn run(&self) -> Result<(), anyhow::Error> {
        let vers = Version::list_local_version()?;
        if vers.is_empty() {
            println!(
                "No Go is installed by goup.{}",
                if let Ok(go_bin) = which("go") {
                    format!(" Using system Go {}.", go_bin.to_string_lossy())
                } else {
                    "".to_owned()
                }
            );
        } else {
            let mut table = Table::new();

            table.add_row(row!["Version", "Active"]);
            for v in vers {
                table.add_row(row![v.version, bc -> if v.active { "*" } else { "" }]);
            }
            table.printstd()
        }
        Ok(())
    }
}
