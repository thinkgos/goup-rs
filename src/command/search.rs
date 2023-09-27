use clap::Args;

use super::Run;

#[derive(Args, Debug)]
pub struct Search;

impl Run for Search {
    fn run(&self) -> Result<(), anyhow::Error> {
        todo!()
    }
}
