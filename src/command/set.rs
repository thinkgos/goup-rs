use clap::Args;
use dialoguer::{theme::ColorfulTheme, Select};

use crate::pkg::version::Version;

use super::Run;

#[derive(Args, Debug)]
#[command(disable_version_flag = true)]
pub struct Set {
    /// target go version
    version: Option<String>,
}

impl Run for Set {
    fn run(&self) -> Result<(), anyhow::Error> {
        if let Some(version) = &self.version {
            Version::switch_go_version(version)
        } else {
            let vers = Version::list_local_version()?;
            let mut items = Vec::new();
            let mut pos = 0;
            for (i, v) in vers.iter().enumerate() {
                items.push(v.version.as_ref());
                if v.active == true {
                    pos = i;
                }
            }
            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select a version")
                .items(&items)
                .default(pos)
                .interact()?;
            Version::switch_go_version(items[selection])
        }
    }
}
