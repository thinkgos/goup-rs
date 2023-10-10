use anyhow::anyhow;
use clap::Args;
// use crate::command::switch_version;

use crate::pkg::dir::Dir;

use super::Run;

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
            install_go_tip(self.cl.as_deref())?;
        } else {
            install_go_version(&version)?;
        }
        // switch_version(&version)
        Ok(())
    }
}

fn get_latest_go_version() -> Result<String, anyhow::Error> {
    Ok("go1.21.1".to_owned())
}

fn install_go_version(version: &str) -> Result<(), anyhow::Error> {
    let home = dirs::home_dir().ok_or_else(|| anyhow!("where is home"))?;
    let target_dir = Dir::new(&home).version(&version);

    if Dir::new(&home)
        .version_dot_unpacked_success(&version)
        .exists()
    {
        println!(
            "{}: already installed in {:?}",
            version,
            target_dir.display()
        );
        return Ok(());
    }

    Ok(())
}

fn install_go_tip(_cl: Option<&str>) -> Result<(), anyhow::Error> {
    Ok(())
}
