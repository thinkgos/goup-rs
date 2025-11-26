use anyhow::anyhow;
use clap::Args;

use super::Run;
use crate::{
    command::utils::InstallOptions,
    registry::{NightlyRegistry, Registry, RegistryIndex},
    toolchain::{self, Toolchain, ToolchainFilter},
    version::Version,
};

#[derive(Args, Debug, PartialEq)]
#[command(disable_version_flag = true)]
pub struct Install {
    /// toolchain name, such as 'stable', 'nightly'('tip', 'gotip'), 'unstable', 'beta' or '=1.21.4'
    #[arg(default_value = "stable", value_parser = clap::value_parser!(Toolchain))]
    toolchain: Toolchain,
    /// an optional change list (CL), If the version is 'tip'
    cl: Option<String>,
    /// only install the version, but do not switch.
    #[arg(long)]
    dry: bool,
    /// use raw version, disable semver, toolchain name such as '1.21.4'
    #[arg(long)]
    pub use_raw_version: bool,
    #[command(flatten)]
    install_options: InstallOptions,
}

impl Run for Install {
    fn run(&self) -> Result<(), anyhow::Error> {
        let opt = &self.install_options;
        let registry_index = RegistryIndex::new(&opt.registry_index);
        let registry = Registry::new(
            &opt.registry,
            opt.skip_verify,
            opt.enable_check_archive_size,
        );
        let version = match self.toolchain {
            Toolchain::Stable => {
                let version = registry_index.get_upstream_latest_go_version()?;
                let version = toolchain::normalize(&version);
                registry.install_go(&version)?;
                version
            }
            Toolchain::Unstable => {
                let version = registry_index
                    .list_upstream_go_versions_filter(Some(&ToolchainFilter::Unstable))?;
                let version = version
                    .last()
                    .ok_or_else(|| anyhow!("failed get latest unstable version"))?;
                let version = toolchain::normalize(version);
                registry.install_go(&version)?;
                version
            }
            Toolchain::Beta => {
                let version = registry_index
                    .list_upstream_go_versions_filter(Some(&ToolchainFilter::Beta))?;
                let version = version
                    .last()
                    .ok_or_else(|| anyhow!("failed get latest beta version"))?;
                let version = toolchain::normalize(version);
                registry.install_go(&version)?;
                version
            }
            Toolchain::Version(ref version_req) => {
                let version = if self.use_raw_version {
                    version_req.to_owned()
                } else {
                    registry_index.match_version_req( version_req).inspect_err(|_| {
                        log::warn!("'semver' match failure, If you want to use version like '1.19beta1' or '1.25rc2', try add option '--use-raw-version'");
                    })?
                };
                let version = toolchain::normalize(&version);
                registry.install_go(&version)?;
                version
            }
            Toolchain::Nightly => {
                log::info!("Installing gotip ...");
                NightlyRegistry::new(self.cl.as_deref()).install_go()?;
                "gotip".to_owned()
            }
        };
        if !self.dry {
            Version::set_go_version(&version)?;
        }
        Ok(())
    }
}
