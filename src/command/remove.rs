use std::fs;

use anyhow::anyhow;
use clap::Args;

use crate::pkg::dir::Dir;

use super::Run;

#[derive(Args, Debug)]
#[command(disable_version_flag = true)]
pub struct Remove {
    /// target go version
    version: Vec<String>,
}

impl Run for Remove {
    fn run(&self) -> Result<(), anyhow::Error> {
        if self.version.is_empty() {
            return Err(anyhow!("No version is specified"));
        }
        let home = dirs::home_dir().ok_or_else(|| anyhow!("where is home"))?;

        for ver in &self.version {
            let version = if ver.starts_with("go") {
                ver.to_string()
            } else {
                format!("go{}", ver)
            };

            println!("Removing {}", version);
            let version_dir = Dir::new(&home).version(&version);
            if version_dir.exists() {
                fs::remove_dir_all(&version_dir)?;
            }
        }
        Ok(())
    }
}
