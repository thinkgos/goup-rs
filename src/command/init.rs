use clap::Args;
use clap_complete::Shell;
use std::env;

use super::Run;
use crate::version::Version;

const SETUP_ENV_UNIX: &str = include_str!("../assets/setup_env_unix");
const SETUP_ENV_FISH: &str = include_str!("../assets/setup_env_fish");

#[derive(Args, Debug, PartialEq)]
pub struct Init {
    /// name of the environment file.
    #[arg(long, default_value = "env")]
    name: String,
    /// detect the shell environment, if not provided, it will be autodetected from the current environment.
    #[arg(value_enum)]
    pub shell: Option<Shell>,
}

impl Run for Init {
    fn run(&self) -> Result<(), anyhow::Error> {
        if let Some(shell_script) = Self::get_shell_setup_script(self.shell) {
            Version::init_env(&self.name, shell_script)?;
        } else {
            log::error!("Unsupported shell setup script.");
        }
        Ok(())
    }
}

impl Init {
    /// Detect the current shell and return the appropriate setup script
    fn get_shell_setup_script(shell: Option<Shell>) -> Option<&'static str> {
        let shell = shell.unwrap_or_else(|| {
            if cfg!(windows) {
                Shell::PowerShell
            } else if let Ok(shell) = env::var("SHELL")
                && shell.contains("fish")
            // Check if SHELL environment variable is set
            {
                Shell::Fish
            } else {
                Shell::Zsh
            }
        });
        match shell {
            Shell::Fish => {
                log::info!("Using fish setup script");
                Some(SETUP_ENV_FISH)
            }
            Shell::PowerShell => None,
            _ => {
                // Fallback to POSIX shell script (bash, zsh, etc.)
                log::info!("Using POSIX shell setup script");
                Some(SETUP_ENV_UNIX)
            }
        }
    }
}
