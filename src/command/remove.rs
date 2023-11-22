use std::fs;

use clap::Args;
use dialoguer::{theme::ColorfulTheme, MultiSelect};

use crate::{dir::Dir, version::Version};

use super::Run;

#[derive(Args, Debug, PartialEq)]
#[command(disable_version_flag = true)]
pub struct Remove {
    /// target go version
    version: Vec<String>,
}

impl Run for Remove {
    fn run(&self) -> Result<(), anyhow::Error> {
        if self.version.is_empty() {
            let vers = Version::list_local_version()?;
            if vers.is_empty() {
                println!("No go is installed");
                return Ok(());
            }
            let items: Vec<&str> = vers.iter().map(|v| v.version.as_ref()).collect();
            let selection = MultiSelect::with_theme(&ColorfulTheme::default())
                .with_prompt("Select multiple version")
                .items(&items)
                .interact()?;
            if selection.is_empty() {
                println!("No item selected");
                return Ok(());
            }
            Self::remove_items(
                &selection
                    .into_iter()
                    .map(|i| items[i])
                    .collect::<Vec<&str>>(),
            )
        } else {
            Self::remove_items(
                &self
                    .version
                    .iter()
                    .map(AsRef::as_ref)
                    .collect::<Vec<&str>>(),
            )
        }
    }
}

impl Remove {
    fn remove_items(vers: &[&str]) -> Result<(), anyhow::Error> {
        let home: std::path::PathBuf = Dir::home_dir()?;
        for ver in vers {
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
