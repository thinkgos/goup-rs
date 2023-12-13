use clap::Args;

use super::Run;
use goup_version::Version;

#[derive(Args, Debug, PartialEq)]
pub struct Search {
    /// a regexp filter
    regex: Option<String>,
}

impl Run for Search {
    fn run(&self) -> Result<(), anyhow::Error> {
        Version::list_upstream_versions(self.regex.as_deref())?
            .iter()
            .for_each(|v| {
                println!("{}", v);
            });
        Ok(())
    }
}
