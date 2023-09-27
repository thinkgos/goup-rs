use clap::Args;

use super::Run;

#[derive(Args, Debug)]
pub struct Install;

impl Run for Install {
    fn run(&self) -> Result<(), anyhow::Error> {
        todo!()
    }
}
