use clap::Args;

use crate::pkg::dir::Dir;

use super::{get_latest_go_version, /*  switch_go_version,*/ Run};

#[derive(Args, Debug)]
#[command(disable_version_flag = true)]
pub struct Install {
    /// target go version
    version: Option<String>,
    /// an optional change list (CL), If the version is 'tip'
    cl: Option<String>,
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
            get_latest_go_version()?
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

        Ok(())
    }
}
