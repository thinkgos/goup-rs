use clap::Args;

use goup_consts::consts;
use goup_downloader::Downloader;
use goup_downloader::Toolchain;
use goup_version::Version;

use super::Run;

#[derive(Args, Debug, PartialEq)]
#[command(disable_version_flag = true)]
pub struct Install {
    /// toolchain name, such as 'stable', 'nightly'('tip', 'gotip',), beta, '1.21.4' or 'go1.21.4'
    #[arg(default_value = "stable")]
    toolchain: String,
    /// an optional change list (CL), If the version is 'tip'
    cl: Option<String>,
    /// host that is used to download Go.
    #[arg(long, default_value_t = consts::GO_HOST.to_owned(), env = consts::GOUP_GO_HOST)]
    host: String,
    /// only just install the version, but do not switch.
    #[arg(long, default_value_t = false)]
    dry: bool,
}

impl Run for Install {
    fn run(&self) -> Result<(), anyhow::Error> {
        let toolchain = self.toolchain.parse()?;

        let version = match toolchain {
            Toolchain::Nightly => {
                println!("Installing gotip ...");
                Downloader::install_go_tip(self.cl.as_deref())?;
                "gotip".to_owned()
            }
            _ => {
                // TODO: beta stable semver
                let version = if let Toolchain::Semver(s) = toolchain {
                    Version::normalize(&s)
                } else {
                    Version::get_upstream_latest_go_version(&self.host)?
                };
                println!("Installing {} ...", version);
                Downloader::install_go_version(&version)?;
                version
            }
        };
        if !self.dry {
            Version::set_go_version(&version)?;
        }
        Ok(())
    }
}
