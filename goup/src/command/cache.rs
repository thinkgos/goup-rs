use clap::Args;
use clap::Subcommand;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Confirm;
use goup_version::Version;

use super::Run;

#[derive(Args, Debug, PartialEq)]
pub struct Cache {
    /// the download command.
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Clone, Debug, PartialEq)]
enum Command {
    /// Show download archive file
    Show(Show),
    /// Clean download archive file
    Clean(Clean),
}

#[derive(Args, Clone, Debug, PartialEq)]
struct Show {
    /// Contain archive sha256 file
    #[arg(short, long, default_value_t = false)]
    contain_sha256: bool,
}

#[derive(Args, Clone, Debug, PartialEq)]
struct Clean {
    /// Skip interact prompt.
    #[arg(short, long, default_value_t = false)]
    no_confirm: bool,
}

impl Run for Cache {
    fn run(&self) -> Result<(), anyhow::Error> {
        match self.command {
            Command::Show(ref arg) => {
                Version::list_cache(Some(arg.contain_sha256))?
                    .iter()
                    .for_each(|v| {
                        println!("{}", v);
                    });
            }
            Command::Clean(ref arg) => {
                let confirmation = arg.no_confirm
                    || Confirm::with_theme(&ColorfulTheme::default())
                        .with_prompt("Do you want to clean cache file?")
                        .interact()?;
                if confirmation {
                    Version::remove_cache()?;
                } else {
                    log::info!("Cancelled");
                }
            }
        }
        Ok(())
    }
}
