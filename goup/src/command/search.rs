use clap::Args;

use super::Run;
use goup_version::consts;
use goup_version::Version;

#[derive(Args, Debug, PartialEq)]
pub struct Search {
    /// a filter, such as 'stable', "unstable", 'beta' or any regex string.
    filter: Option<String>,
    /// host that is used to download Go.
    #[arg(long, default_value_t = consts::GO_HOST.to_owned(), env = consts::GOUP_GO_HOST)]
    host: String,
    /// use git, default use http.
    #[arg(long, default_value_t = false)]
    use_git: bool,
}

impl Run for Search {
    fn run(&self) -> Result<(), anyhow::Error> {
        let filter = self.filter.as_ref().and_then(|s| s.parse().ok());
        let vers = if self.use_git {
            Version::list_upstream_go_versions_from_git(filter)
        } else {
            Version::list_upstream_go_versions_from_http(&self.host, filter)
        };
        vers?.iter().for_each(|v| {
            println!("{}", v);
        });
        Ok(())
    }
}
