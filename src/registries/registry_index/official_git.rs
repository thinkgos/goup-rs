use std::process::Command;

use anyhow::anyhow;
use regex::Regex;
use which::which;

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
        if which("git").is_err() {
            return Err(anyhow!(
                r#""git" binary not found, make sure it is installed!"#,
            ));
        }

        let output = Command::new("git")
            .args(["ls-remote", "--sort=version:refname", "--tags", &self.url])
            .output()?
            .stdout;
        let versions: Vec<String> = Regex::new("refs/tags/go(.+)")?
            .captures_iter(&String::from_utf8_lossy(&output))
            .map(|capture| capture[1].to_string())
            .collect();
        Ok(versions.into())

        // miss --tags
        // let mut remote = Remote::create_detached(self.url.clone())?;
        // remote.connect(Direction::Fetch)?;
        // let refs = remote.list()?;
        // let re: Regex = Regex::new(r"refs/tags/go(.+)")?;
        // let versions = refs
        //     .iter()
        //     .filter_map(|r| re.captures(r.name()))
        //     .map(|caps| caps[1].to_string())
        //     .collect::<Vec<_>>();
        // remote.disconnect()?;
        // Ok(versions.into())
    }
}
