use clap::Args;

use super::Run;
use crate::{consts, registry::RegistryIndex};

#[derive(Args, Debug, PartialEq)]
pub struct Search {
    /// a filter, such as 'stable', "unstable", 'beta' or any regex string(1.22.*).
    filter: Option<String>,
    /// registry index that is used to update Go version index.
    #[arg(long, default_value_t = consts::GO_REGISTRY_INDEX.to_owned(), env = consts::GO_REGISTRY_INDEX)]
    registry_index: String,
}

impl Run for Search {
    fn run(&self) -> Result<(), anyhow::Error> {
        let filter = self.filter.as_ref().and_then(|s| s.parse().ok());
        let registry_index = RegistryIndex::new(&self.registry_index);
        registry_index
            .list_upstream_go_versions_filter(filter)?
            .iter()
            .for_each(|v| {
                println!("{v}");
            });
        Ok(())
    }
}
