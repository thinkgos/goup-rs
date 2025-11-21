use std::collections::HashSet;

use clap::Args;
use owo_colors::OwoColorize;

use super::Run;
use crate::{consts, registry::RegistryIndex, version::Version};

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
        let remote_versions =
            RegistryIndex::new(&self.registry_index).list_upstream_go_versions_filter(filter)?;
        let local_versions = Version::list_go_version().unwrap_or_default();
        let local_versions: HashSet<_> = local_versions.iter().map(|v| &v.version).collect();
        remote_versions.iter().for_each(|version| {
            if local_versions.contains(&version) {
                println!("{}{}", version.yellow(), "(installed)".yellow());
            } else {
                println!("{version}");
            }
        });
        Ok(())
    }
}
