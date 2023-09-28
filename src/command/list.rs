use clap::Args;
use prettytable::{row, Table};

use super::Run;
use crate::pkg::go_version::{show_go_if_exist, GoVersion};

#[derive(Args, Debug)]
pub struct List;

impl Run for List {
    fn run(&self) -> Result<(), anyhow::Error> {
        let vers = GoVersion::list()?;
        if vers.is_empty() {
            show_go_if_exist()
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
