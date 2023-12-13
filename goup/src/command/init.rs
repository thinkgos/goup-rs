use clap::Args;
use goup_version::Version;

use super::Run;

const SETUP_ENV_UNIX: &str = include_str!("../../../setup_env_unix");

#[derive(Args, Debug, PartialEq)]
pub struct Init;

impl Run for Init {
    fn run(&self) -> Result<(), anyhow::Error> {
        Version::init_env(SETUP_ENV_UNIX)?;
        Ok(())
    }
}
