use anyhow::anyhow;
use clap::Args;
use dialoguer::{Select, theme::ColorfulTheme};

use crate::{command::utils::InstallOptions, registry::Registry, toolchain, version::Version};

use super::Run;

#[derive(Args, Debug, PartialEq)]
#[command(disable_version_flag = true)]
pub struct Set {
    /// target go version
    version: Option<String>,
    #[command(flatten)]
    install_options: InstallOptions,
}

impl Run for Set {
    fn run(&self) -> Result<(), anyhow::Error> {
        let vers = Version::list_go_version()?;
        if vers.is_empty() {
            return Err(anyhow!(
                "Not any go is installed, Install it with `goup install`."
            ));
        }
        if let Some(version) = &self.version {
            if !vers.iter().any(|v| v.version == *version) {
                let registry = Registry::new(
                    &self.install_options.registry,
                    self.install_options.skip_verify,
                    self.install_options.enable_check_archive_size,
                );
                let version = toolchain::normalize(version);
                registry.install_go(&version)?
            }
            Version::set_go_version(version)
        } else {
            let mut items = Vec::new();
            let mut pos = 0;
            for (i, v) in vers.iter().enumerate() {
                items.push(v.version.as_ref());
                if v.default {
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
