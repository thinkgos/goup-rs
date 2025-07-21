use clap::Args;
use dialoguer::{MultiSelect, theme::ColorfulTheme};

use crate::version::Version;

use super::Run;

#[derive(Args, Debug, PartialEq)]
#[command(disable_version_flag = true)]
pub struct Remove {
    /// target go version list.
    version: Vec<String>,
}

impl Run for Remove {
    fn run(&self) -> Result<(), anyhow::Error> {
        if self.version.is_empty() {
            let vers = Version::list_go_version()?;
            if vers.is_empty() {
                log::info!("No go is installed");
                return Ok(());
            }
            let items: Vec<&str> = vers.iter().map(|v| v.version.as_ref()).collect();
            let selection = MultiSelect::with_theme(&ColorfulTheme::default())
                .with_prompt("Select multiple version")
                .items(&items)
                .interact()?;
            if selection.is_empty() {
                log::info!("No item selected");
                return Ok(());
            }

            let vers = selection
                .into_iter()
                .map(|i| items[i])
                .collect::<Vec<&str>>();
            Version::remove_go_versions(&vers)
        } else {
            let vers = self
                .version
                .iter()
                .map(AsRef::as_ref)
                .collect::<Vec<&str>>();
            Version::remove_go_versions(&vers)
        }
    }
}
