use anyhow::anyhow;
use clap::Args;
use dialoguer::{Select, theme::ColorfulTheme};

use crate::{
    command::utils::InstallOptions, registries::registry::Registry, toolchain, version::Version,
};

use super::Run;

#[derive(Args, Debug, PartialEq)]
#[command(disable_version_flag = true)]
pub struct Default {
    /// target go version
    version: Option<String>,
    #[command(flatten)]
    install_options: InstallOptions,
}

impl Run for Default {
    fn run(&self) -> Result<(), anyhow::Error> {
        let versions = Version::list_go_version()?;
        let target_version = if let Some(version) = &self.version {
            if !versions.iter().any(|v| v.version == *version) {
                let registry = Registry::new(
                    &self.install_options.registry,
                    self.install_options.skip_verify,
                    self.install_options.enable_check_archive_size,
                );
                registry.install_go(&toolchain::normalize(version))?
            }
            version
        } else {
            if versions.is_empty() {
                return Err(anyhow!(
                    "Not any go is installed, Install it with `goup install`."
                ));
            }
            let mut items = Vec::new();
            let mut pos = 0;
            for (i, v) in versions.iter().enumerate() {
                items.push(&v.version);
                if v.default {
                    pos = i;
                }
            }
            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select a version")
                .items(&items)
                .default(pos)
                .interact()?;
            items[selection]
        };
        Version::set_go_version(target_version)
    }
}
