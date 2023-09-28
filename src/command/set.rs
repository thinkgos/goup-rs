use std::fs;

use anyhow::anyhow;
use clap::Args;
// use ratatui::prelude::CrosstermBackend;
// use ratatui::terminal::Terminal;

use super::Run;
use crate::pkg::dir::Dir;

#[derive(Args, Debug)]
#[command(disable_version_flag = true)]
pub struct Set {
    /// target go version
    version: Option<String>,
}

impl Run for Set {
    fn run(&self) -> Result<(), anyhow::Error> {
        if let Some(version) = &self.version {
            let version = if version.starts_with("go") {
                version.to_string()
            } else {
                format!("go{}", version)
            };
            let home = dirs::home_dir().ok_or_else(|| anyhow!("where is home"))?;

            let version_dir = Dir::new(&home).version(&version);
            if !version_dir.exists() {
                return Err(anyhow!(
                    "Go version {version} is not installed. Install it with `goup install`."
                ));
            }
            let current = Dir::new(&home).current();
            fs::remove_file(&current)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs as unix_fs;
                unix_fs::symlink(&version_dir, &current)?;
            }
            #[cfg(windows)]
            {
                use std::os::windows::fs as windows_fs;
                windows_fs::symlink_file(&version_dir, &current)?;
            }
            println!("Default Go is set to '{version}'")
        } else {
            // let mut term = Terminal::new(CrosstermBackend::new(io::stdout()))?;
            // _ = term
        }
        Ok(())
    }
}
