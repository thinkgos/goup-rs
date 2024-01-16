use clap::Args;

use goup_consts::consts;
use goup_downloader::Downloader;
use goup_version::Version;

use super::Run;

#[derive(Args, Debug, PartialEq)]
#[command(disable_version_flag = true)]
pub struct Install {
    /// target go version
    version: Option<String>,
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
        let version = if let Some(s) = self.version.as_deref() {
            Version::normalize(s)
        } else {
            Version::get_upstream_latest_go_version(&self.host)?
        };
        println!("Installing {} ...", version);
        if version == "gotip" {
            Downloader::install_go_tip(self.cl.as_deref())?;
        } else {
            Downloader::install_go_version(&version)?;
        }
        if !self.dry {
            Version::set_go_version(&version)?;
        }
        Ok(())
    }
}
