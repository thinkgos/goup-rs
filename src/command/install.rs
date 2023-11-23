use clap::Args;

use govm_consts::consts;
use govm_downloader::Downloader;
use govm_version::Version;

use super::Run;

#[derive(Args, Debug, PartialEq)]
#[command(disable_version_flag = true)]
pub struct Install {
    /// target go version
    version: Option<String>,
    /// an optional change list (CL), If the version is 'tip'
    cl: Option<String>,
    /// host that is used to download Go.
    #[arg(long, default_value_t = consts::GO_HOST.to_owned(), env = consts::GOVM_GO_HOST)]
    host: String,
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
        Version::set_go_version(&version)
    }
}
