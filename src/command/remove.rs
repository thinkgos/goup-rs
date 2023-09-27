use clap::Args;

use super::Run;

#[derive(Args, Debug)]
pub struct Remove;

impl Run for Remove {
    fn run(&self) -> Result<(), anyhow::Error> {
        todo!()
    }
}
