use std::collections::HashSet;

use clap::Args;
use owo_colors::OwoColorize;

use super::Run;
use crate::registries::registry_index::RegistryIndex;
use crate::{consts, toolchain::ToolchainFilter, version::Version};

#[derive(Args, Debug, PartialEq)]
pub struct Search {
    /// a filter, such as 'stable', "unstable", 'beta' or any regex string(1.22.*).
    #[arg(value_parser = clap::value_parser!(ToolchainFilter))]
    filter: Option<ToolchainFilter>,
    /// registry index that is used to update Go version index.
    #[arg(long, default_value_t = consts::GO_REGISTRY_INDEX.to_owned(), env = consts::GO_REGISTRY_INDEX)]
    registry_index: String,
}

impl Run for Search {
    fn run(&self) -> Result<(), anyhow::Error> {
        let remote_versions = RegistryIndex::new(&self.registry_index)
            .list_upstream_go_versions_filter(self.filter.as_ref())?;
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
