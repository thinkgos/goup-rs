use anyhow::anyhow;
use clap::Args;
use dialoguer::{Select, theme::ColorfulTheme};

use crate::version::Version;

use super::Run;

#[derive(Args, Debug, PartialEq)]
#[command(disable_version_flag = true)]
pub struct Set {
    /// target go version
    version: Option<String>,
}

impl Run for Set {
    fn run(&self) -> Result<(), anyhow::Error> {
        if let Some(version) = &self.version {
            Version::set_go_version(version)
        } else {
            let vers = Version::list_go_version()?;
            if vers.is_empty() {
                return Err(anyhow!(
                    "Not any go is installed, Install it with `goup install`."
                ));
            }

            let mut items = Vec::new();
            let mut pos = 0;
            for (i, v) in vers.iter().enumerate() {
                items.push(v.version.as_ref());
                if v.active {
                    pos = i;
                }
            }
            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select a version")
                .items(&items)
                .default(pos)
                .interact()?;
            Version::set_go_version(items[selection])
        }
    }
}
