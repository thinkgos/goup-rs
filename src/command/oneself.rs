use std::env;
use std::fs;
use std::path::Path;

use anyhow::anyhow;
use clap::Args;
use clap::CommandFactory;
use clap::Subcommand;
use dialoguer::Confirm;
use dialoguer::theme::ColorfulTheme;
use self_update::{backends::github::Update, cargo_crate_version};

use super::Cli;
use super::Run;
use crate::version::Version;

#[derive(Args, Debug, PartialEq)]
pub struct Oneself {
    /// the goup installation command.
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Clone, Debug, PartialEq)]
enum Command {
    /// Download and install updates to goup
    Update,
    // Uninstall goup
    Uninstall(Uninstall),
}

#[derive(Args, Clone, Debug, PartialEq)]
struct Uninstall {
    /// Skip interact prompt.
    #[arg(short, long)]
    no_confirm: bool,
}

impl Run for Oneself {
    fn run(&self) -> Result<(), anyhow::Error> {
        match self.command {
            Command::Update => {
                if cfg!(feature = "no-self-update") {
                    log::warn!("self-update is disabled for this build of goup");
                    log::warn!("you should use your system package manager to update goup");
                    return Ok(());
                }
                let cmd = Cli::command();
                let status = Update::configure()
                    .repo_owner("thinkgos")
                    .repo_name("goup-rs")
                    .identifier(self_update_asset_identifier())
                    .bin_name(cmd.get_name())
                    .show_download_progress(true)
                    .no_confirm(true)
                    .current_version(cargo_crate_version!())
                    .build()?
                    .update()?;
                log::info!("Update status: `v{}`!", status.version());
            }
            Command::Uninstall(ref arg) => {
                if cfg!(feature = "no-self-update") {
                    log::warn!("self-uninstall is disabled for this build of goup");
                    log::warn!("you should use your system package manager to uninstall goup");
                    return Ok(());
                }
                let confirmation = arg.no_confirm
                    || Confirm::with_theme(&ColorfulTheme::default())
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

fn self_update_asset_identifier() -> &'static str {
    #[cfg(windows)]
    {
        ".zip"
    }
    #[cfg(not(windows))]
    {
        ".tar.gz"
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
