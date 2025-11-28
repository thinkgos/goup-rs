use std::process::Command;

use regex::Regex;

use crate::registries::{go_index::GoIndex, registry_index::RegistryIndex};

#[derive(Debug)]
pub struct OfficialGit {
    url: String,
}

impl RegistryIndex for OfficialGit {
    fn get_upstream_latest_go_version(&self) -> Result<String, anyhow::Error> {
        self.inner_list_upstream_go_versions().map(|i| i.latest)
    }
    fn list_upstream_go_versions(&self) -> Result<Vec<String>, anyhow::Error> {
        self.inner_list_upstream_go_versions().map(|i| i.versions)
    }
}

impl OfficialGit {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_owned(),
        }
    }
    fn inner_list_upstream_go_versions(&self) -> Result<GoIndex, anyhow::Error> {
        let output = Command::new("git")
            .args(["ls-remote", "--sort=version:refname", "--tags", &self.url])
            .output()?
            .stdout;
        let versions: Vec<String> = Regex::new("refs/tags/go(.+)")?
            .captures_iter(&String::from_utf8_lossy(&output))
            .map(|capture| capture[1].to_string())
            .collect();
        Ok(versions.into())
    }
}
