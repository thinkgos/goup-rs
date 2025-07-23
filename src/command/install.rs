use anyhow::anyhow;
use clap::Args;

use crate::version::Version;
use crate::version::consts;
use crate::version::toolchain::Toolchain;
use crate::version::toolchain::ToolchainFilter;

use super::Run;
use crate::downloader::Downloader;

#[derive(Args, Debug, PartialEq)]
#[command(disable_version_flag = true)]
pub struct Install {
    /// toolchain name, such as 'stable', 'nightly'('tip', 'gotip'), 'unstable', 'beta' or '=1.21.4'
    #[arg(default_value = "stable")]
    toolchain: String,
    /// an optional change list (CL), If the version is 'tip'
    cl: Option<String>,
    /// host that is used to download Go.
    #[arg(long, default_value_t = consts::GO_HOST.to_owned(), env = consts::GOUP_GO_HOST)]
    host: String,
    /// only install the version, but do not switch.
    #[arg(long, default_value_t = false)]
    dry: bool,
    /// skip sha256 verification.
    #[arg(long, default_value_t = false)]
    skip_verify: bool,
    /// use raw version, disable semver, toolchain name such as '1.21.4'
    #[arg(long, default_value_t = false)]
    use_raw_version: bool,
}

impl Run for Install {
    fn run(&self) -> Result<(), anyhow::Error> {
        let toolchain = self.toolchain.parse()?;
        let version = match toolchain {
            Toolchain::Stable => {
                let version = Version::get_upstream_latest_go_version(&self.host)?;
                let version = Version::normalize(&version);
                log::info!("Installing {version} ...");
                Downloader::install_go_version(&version, &self.skip_verify)?;
                version
            }
            Toolchain::Unstable => {
                let version = Version::list_upstream_go_versions_filter(
                    &self.host,
                    Some(ToolchainFilter::Unstable),
                )?;
                let version = version
                    .last()
                    .ok_or_else(|| anyhow!("failed get latest unstable version"))?;
                let version = Version::normalize(version);
                log::info!("Installing {version} ...");
                Downloader::install_go_version(&version, &self.skip_verify)?;
                version
            }
            Toolchain::Beta => {
                let version = Version::list_upstream_go_versions_filter(
                    &self.host,
                    Some(ToolchainFilter::Beta),
                )?;
                let version = version
                    .last()
                    .ok_or_else(|| anyhow!("failed get latest beta version"))?;
                let version = Version::normalize(version);
                log::info!("Installing {version} ...");
                Downloader::install_go_version(&version, &self.skip_verify)?;
                version
            }
            Toolchain::Version(ver_req) => {
                let version = if self.use_raw_version {
                    ver_req
                } else {
                    Version::match_version_req(&self.host, &ver_req).inspect_err(|_| {
                        log::warn!("'semver' parse failure, If you want to use versions like '1.19beta1' or '1.25rc2' (non-standard semantic versions), try add option '--use-raw-version'");
                    })?
                };
                let version = Version::normalize(&version);
                log::info!("Installing {version} ...");
                Downloader::install_go_version(&version, &self.skip_verify)?;
                version
            }
            Toolchain::Nightly => {
                log::info!("Installing gotip ...");
                Downloader::install_go_tip(self.cl.as_deref())?;
                "gotip".to_owned()
            }
        };
        if !self.dry {
            Version::set_go_version(&version)?;
        }
        Ok(())
    }
}
