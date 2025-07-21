use clap::Args;

use super::Run;
use crate::version::Version;
use crate::version::consts;

#[derive(Args, Debug, PartialEq)]
pub struct Search {
    /// a filter, such as 'stable', "unstable", 'beta' or any regex string(1.22.*).
    filter: Option<String>,
    /// host that is used to download Go.
    #[arg(long, default_value_t = consts::GO_HOST.to_owned(), env = consts::GOUP_GO_HOST)]
    host: String,
}

impl Run for Search {
    fn run(&self) -> Result<(), anyhow::Error> {
        let filter = self.filter.as_ref().and_then(|s| s.parse().ok());
        Version::list_upstream_go_versions_filter(&self.host, filter)?
            .iter()
            .for_each(|v| {
                println!("{v}");
            });
        Ok(())
    }
}
