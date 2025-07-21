use clap::Args;
use std::env;

use super::Run;
use crate::version::Version;

const SETUP_ENV_UNIX: &str = include_str!("../../setup_env_unix");
const SETUP_ENV_FISH: &str = include_str!("../../setup_env_fish");

#[derive(Args, Debug, PartialEq)]
pub struct Init;

impl Run for Init {
    fn run(&self) -> Result<(), anyhow::Error> {
        let shell_script = Self::detect_shell_and_get_script();
        Version::init_env(shell_script)?;
        Ok(())
    }
}

impl Init {
    /// Detect the current shell and return the appropriate setup script
    fn detect_shell_and_get_script() -> &'static str {
        // Check if SHELL environment variable is set
        if let Ok(shell) = env::var("SHELL") {
            if shell.contains("fish") {
                log::info!("Detected fish shell, using fish setup script");
                return SETUP_ENV_FISH;
            }
        }

        // Fallback to POSIX shell script (bash, zsh, etc.)
        log::info!("Using POSIX shell setup script");
        SETUP_ENV_UNIX
    }
}
