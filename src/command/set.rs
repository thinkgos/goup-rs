use clap::Args;

use super::Run;

#[derive(Args, Debug)]
pub struct Set;

impl Run for Set {
    fn run(&self) -> Result<(), anyhow::Error> {
        todo!()
    }
}
