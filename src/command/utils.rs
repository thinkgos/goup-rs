use std::str::FromStr;

use clap::Args;

use crate::{consts, registries::registry_index::RegistryIndexType};
#[derive(Args, Debug, PartialEq)]
pub struct InstallOptions {
    /// skip sha256 verification.
    #[arg(long)]
    pub skip_verify: bool,
    /// enable check archive size.
    #[arg(long)]
    pub enable_check_archive_size: bool,
    /// registry index that is used to update Go version index.
    #[arg(long, default_value_t = RegistryIndexType::Official(consts::GO_REGISTRY_INDEX.to_owned()), env = consts::GOUP_GO_REGISTRY_INDEX, value_parser = clap::value_parser!(RegistryIndexType))]
    pub registry_index: RegistryIndexType,
    /// registry that is used to download Go archive file.
    #[arg(long, default_value_t = consts::GO_REGISTRY.to_owned(), env = consts::GOUP_GO_REGISTRY)]
    pub registry: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct KeyValuePair {
    pub key: String,
    pub value: String,
}

impl FromStr for KeyValuePair {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.splitn(2, '=');
        let k = parts
            .next()
            .filter(|v| !v.is_empty())
            .ok_or_else(|| anyhow::anyhow!("expected KEY=VALUE, empty key, got `{}`", s))?;
        let v = parts
            .next()
            .ok_or_else(|| anyhow::anyhow!("expected KEY=VALUE, got `{}`", s))?;
        Ok(KeyValuePair {
            key: k.to_string(),
            value: v.to_string(),
        })
    }
}
