use anyhow::anyhow;
use clap::Args;

use super::Run;
use crate::{
    consts,
    registry::{NightlyRegistry, Registry, RegistryIndex},
    toolchain,
    toolchain::{Toolchain, ToolchainFilter},
    version::Version,
};

#[derive(Args, Debug, PartialEq)]
#[command(disable_version_flag = true)]
pub struct Install {
    /// toolchain name, such as 'stable', 'nightly'('tip', 'gotip'), 'unstable', 'beta' or '=1.21.4'
    #[arg(default_value = "stable")]
    toolchain: String,
    /// an optional change list (CL), If the version is 'tip'
    cl: Option<String>,
    /// only install the version, but do not switch.
    #[arg(long)]
    dry: bool,
    /// skip sha256 verification.
    #[arg(long)]
    skip_verify: bool,
    /// use raw version, disable semver, toolchain name such as '1.21.4'
    #[arg(long)]
    use_raw_version: bool,
    /// registry index that is used to update Go version index.
    #[arg(long, default_value_t = consts::GO_REGISTRY_INDEX.to_owned(), env = consts::GOUP_GO_REGISTRY_INDEX)]
    registry_index: String,
    /// registry that is used to download Go archive file.
    #[arg(long, default_value_t = consts::GO_REGISTRY.to_owned(), env = consts::GOUP_GO_REGISTRY)]
    registry: String,
}

impl Run for Install {
    fn run(&self) -> Result<(), anyhow::Error> {
        let toolchain = self.toolchain.parse()?;
        let registry_index = RegistryIndex::new(&self.registry_index);
        let registry = Registry::new(&self.registry);
        let version = match toolchain {
            Toolchain::Stable => {
                let version = registry_index.get_upstream_latest_go_version()?;
                let version = toolchain::normalize(&version);
                registry.install_go(&version, &self.skip_verify)?;
                version
            }
            Toolchain::Unstable => {
                let version = registry_index
                    .list_upstream_go_versions_filter(Some(ToolchainFilter::Unstable))?;
                let version = version
                    .last()
                    .ok_or_else(|| anyhow!("failed get latest unstable version"))?;
                let version = toolchain::normalize(version);
                registry.install_go(&version, &self.skip_verify)?;
                version
            }
            Toolchain::Beta => {
                let version =
                    registry_index.list_upstream_go_versions_filter(Some(ToolchainFilter::Beta))?;
                let version = version
                    .last()
                    .ok_or_else(|| anyhow!("failed get latest beta version"))?;
                let version = toolchain::normalize(version);
                registry.install_go(&version, &self.skip_verify)?;
                version
            }
            Toolchain::Version(ver_req) => {
                let version = if self.use_raw_version {
                    ver_req
                } else {
                    registry_index.match_version_req( &ver_req).inspect_err(|_| {
                        log::warn!("'semver' parse failure, If you want to use versions like '1.19beta1' or '1.25rc2' (non-standard semantic versions), try add option '--use-raw-version'");
                    })?
                };
                let version = toolchain::normalize(&version);
                registry.install_go(&version, &self.skip_verify)?;
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
