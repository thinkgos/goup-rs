use std::time::Duration;

use anyhow::anyhow;
use reqwest::blocking::Client;
use serde::Deserialize;
use serde::Serialize;

use crate::registries::registry_index::RegistryIndex;

const HTTP_TIMEOUT: Duration = Duration::from_secs(10);

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct GoFile {
    pub arch: String,
    pub filename: String,
    pub kind: String,
    pub os: String,
    pub sha256: String,
    pub size: isize,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct GoRelease {
    pub version: String,
    pub stable: bool,
    // pub files: Vec<GoFile>,
}

#[derive(Debug, Clone)]
pub struct Official {
    host: String,
}

impl RegistryIndex for Official {
    /// get upstream latest go version.
    fn get_upstream_latest_go_version(&self) -> Result<String, anyhow::Error> {
        let body = Client::builder()
            .timeout(HTTP_TIMEOUT)
            .build()?
            .get(format!("{}/VERSION?m=text", self.host))
            .send()?
            .text()?;
        body.split('\n')
            .nth(0)
            .ok_or_else(|| anyhow!("Getting latest Go version failed"))
            .map(|v| v.to_owned())
    }

    /// list upstream go versions.
    fn list_upstream_go_versions(&self) -> Result<Vec<String>, anyhow::Error> {
        Ok(Client::builder()
            .timeout(HTTP_TIMEOUT)
            .build()?
            .get(format!("{}/dl/?mode=json&include=all", self.host))
            .send()?
            .json::<Vec<GoRelease>>()?
            .into_iter()
            .map(|v| v.version.trim_start_matches("go").to_string())
            .rev()
            .collect())
    }
}

impl Official {
    pub fn new(host: &str) -> Self {
        Self {
            host: host.to_owned(),
        }
    }
}
