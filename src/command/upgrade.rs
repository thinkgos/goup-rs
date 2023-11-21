use clap::Args;
use clap::CommandFactory;
use self_update::{backends::github::Update, cargo_crate_version};

use super::Cli;
use super::Run;

#[derive(Args, Debug, PartialEq)]
pub struct Upgrade;

impl Run for Upgrade {
    fn run(&self) -> Result<(), anyhow::Error> {
        let cmd = Cli::command();
        let status = Update::configure()
            .repo_owner("thinkgos")
            .repo_name("govm")
            .bin_name(cmd.get_name())
            .show_download_progress(true)
            .no_confirm(true)
            .current_version(cargo_crate_version!())
            .build()?
            .update()?;
        println!("Update status: `{}`!", status.version());
        Ok(())
    }
}
