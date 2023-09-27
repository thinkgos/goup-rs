use clap::Args;

use super::Run;

#[derive(Args, Debug)]
pub struct List;

impl Run for List {
    fn run(&self) -> Result<(), anyhow::Error> {
        todo!()
    }
}
