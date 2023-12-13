use clap::Args;
use dialoguer::{theme::ColorfulTheme, MultiSelect};

use goup_version::Version;

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
            let vers = Version::list_go_version()?;
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
            Version::remove_go_versions(
                &selection
                    .into_iter()
                    .map(|i| items[i])
                    .collect::<Vec<&str>>(),
            )
        } else {
            Version::remove_go_versions(
                &self
                    .version
                    .iter()
                    .map(AsRef::as_ref)
                    .collect::<Vec<&str>>(),
            )
        }
    }
}
