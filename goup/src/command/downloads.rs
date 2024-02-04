use clap::Args;
use clap::Subcommand;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Confirm;
use goup_version::Version;

use super::Run;

#[derive(Args, Debug, PartialEq)]
pub struct Downloads {
    // outputs the completion content for given shell
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Clone, Debug, PartialEq)]
enum Command {
    /// Show download archive file
    Show(Show),
    /// Clean download archive file
    Clean,
}

#[derive(Args, Clone, Debug, PartialEq)]
struct Show {
    #[arg(short, long, default_value_t = false)]
    contain_sha256: bool,
}

impl Run for Downloads {
    fn run(&self) -> Result<(), anyhow::Error> {
        match self.command {
            Command::Show(ref show) => {
                Version::list_dl(Some(show.contain_sha256))?
                    .iter()
                    .for_each(|v| {
                        println!("{}", v);
                    });
            }
            Command::Clean => {
                let confirmation = Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt("Do you want to clean archive file?")
                    .interact()?;
                if confirmation {
                    Version::remove_dl()?;
                } else {
                    log::info!("Cancelled");
                }
            }
        }
        Ok(())
    }
}
