use std::env;
use std::fs;
use std::path::Path;

use anyhow::anyhow;
use clap::Args;
use clap::CommandFactory;
use clap::Subcommand;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Confirm;
use goup_version::Version;
use self_update::{backends::github::Update, cargo_crate_version};

use super::Cli;
use super::Run;

#[derive(Args, Debug, PartialEq)]
pub struct Oneself {
    // the goup installation command.
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Clone, Debug, PartialEq)]
enum Command {
    /// Download and install updates to goup
    Update,
    // Uninstall goup
    Uninstall,
}

impl Run for Oneself {
    fn run(&self) -> Result<(), anyhow::Error> {
        match self.command {
            Command::Update => {
                let cmd = Cli::command();
                let status = Update::configure()
                    .repo_owner("thinkgos")
                    .repo_name("goup-rs")
                    .bin_name(cmd.get_name())
                    .show_download_progress(true)
                    .no_confirm(true)
                    .current_version(cargo_crate_version!())
                    .build()?
                    .update()?;
                log::info!("Update status: `v{}`!", status.version());
            }
            Command::Uninstall => {
                let confirmation = Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt("Do you want to uninstall goup?")
                    .interact()?;
                if confirmation {
                    remove_goup_exe()?;
                    Version::remove_goup_home()?;
                    log::info!("Uninstall successful!");
                } else {
                    log::info!("Cancelled!");
                }
            }
        }
        Ok(())
    }
}

fn remove_goup_exe() -> Result<(), anyhow::Error> {
    let exe = env::args().next().ok_or(anyhow!("Get exe path Failed"))?;
    let exe = Path::new(&exe);
    if exe.is_symlink() {
        let link_file = exe.read_link()?;
        fs::remove_file(link_file)?;
        fs::remove_dir_all(exe)?;
    } else {
        fs::remove_file(exe)?;
    }
    Ok(())
}
