use anyhow::anyhow;
use clap::Args;
use reqwest::blocking;

use crate::pkg::{consts, dir::Dir};

use super::Run;

#[derive(Args, Debug)]
#[command(disable_version_flag = true)]
pub struct Install {
    /// target go version
    version: Option<String>,
    /// an optional change list (CL), If the version is 'tip'
    cl: Option<String>,
    /// host that is used to download Go.
    #[arg(long, default_value_t = consts::GO_HOST.to_owned(), env = "GOUP_GO_HOST")]
    host: String,
}

impl Run for Install {
    fn run(&self) -> Result<(), anyhow::Error> {
        let version = if let Some(s) = self.version.as_deref() {
            if s.starts_with("go") {
                s.to_owned()
            } else {
                format!("go{}", s)
            }
        } else {
            self.get_latest_go_version()?
        };
        if version == "gotip" {
            self.install_go_tip()?;
        } else {
            self.install_go_version(&version)?;
        }
        // switch_go_version(&version)
        Ok(())
    }
}

impl Install {
    fn install_go_tip(&self) -> Result<(), anyhow::Error> {
        //    self.cl.as_deref()
        Ok(())
    }
    fn install_go_version(&self, version: &str) -> Result<(), anyhow::Error> {
        let home = Dir::home_dir()?;
        let target_dir = Dir::new(&home).version(&version);

        if Dir::is_dot_unpacked_success_exists(&home, &version) {
            println!(
                "{}: already installed in {:?}",
                version,
                target_dir.display()
            );
            return Ok(());
        }
        let go_archive_url = consts::go_version_archive_url(version);
        println!("{go_archive_url}");

        Ok(())
    }
    fn get_latest_go_version(&self) -> Result<String, anyhow::Error> {
        let url = format!("{}/VERSION?m=text", self.host);
        let body = blocking::get(&url)?.text()?;
        let ver = body
            .split("\n")
            .nth(0)
            .ok_or_else(|| anyhow!("Getting latest Go version failed"))?;
        Ok(ver.to_owned())
    }
}
