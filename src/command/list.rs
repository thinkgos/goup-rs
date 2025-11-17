use std::io::{self, Write};

use clap::Args;
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
                    (true, true) => {
                        writeln!(stdout, "{}   (active, default & session)", v.version)?
                    }
                    (true, _) => writeln!(stdout, "{}  (active, default)", v.version)?,
                    (_, true) => writeln!(stdout, "{}  (active, session)", v.version)?,
                    _ => writeln!(stdout, "{}", v.version)?,
                };
            }
            stdout.flush()?;
        }
        Ok(())
    }
}
