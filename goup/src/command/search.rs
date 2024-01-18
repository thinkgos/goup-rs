use clap::Args;

use super::Run;
use goup_version::Version;

#[derive(Args, Debug, PartialEq)]
pub struct Search {
    /// a filter, such as 'stable', "unstable", 'beta' or any regex string.
    filter: Option<String>,
}

impl Run for Search {
    fn run(&self) -> Result<(), anyhow::Error> {
        let filter = self.filter.as_ref().and_then(|s| s.parse().ok());
        Version::list_upstream_go_versions(filter)?
            .iter()
            .for_each(|v| {
                println!("{}", v);
            });
        Ok(())
    }
}
