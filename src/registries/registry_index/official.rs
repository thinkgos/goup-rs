use std::{process::Command, sync::mpsc, thread, time::Duration};

use anyhow::anyhow;
use regex::Regex;
use reqwest::blocking::Client;
use serde::Deserialize;
use serde::Serialize;
use which::which;

use crate::consts;
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
        let (tx, rx) = mpsc::channel();
        {
            let tx = tx.clone();
            let this = self.clone();
            thread::spawn(move || {
                let r = this.list_upstream_go_versions_via_http();
                let _ = tx.send(r);
            });
        }

        let thread_count = if which("git").is_ok() {
            let tx = tx.clone();
            thread::spawn(move || {
                let r: Result<Vec<String>, anyhow::Error> =
                    Self::list_upstream_go_versions_via_git();
                let _ = tx.send(r);
            });
            2
        } else {
            1
        };

        let mut last_err = Ok(vec![]);
        for _ in 0..thread_count {
            match rx.recv()? {
                ok @ Ok(_) => return ok,
                Err(e) => last_err = Err(e),
            }
        }
        last_err
    }
}

impl Official {
    pub fn new(host: &str) -> Self {
        Self {
            host: host.to_owned(),
        }
    }
    /// list upstream go versions via http.
    fn list_upstream_go_versions_via_http(&self) -> Result<Vec<String>, anyhow::Error> {
        log::trace!("list upstream go versions via http: {:?}", self.host);
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
    /// list upstream go versions via git.
    fn list_upstream_go_versions_via_git() -> Result<Vec<String>, anyhow::Error> {
        let go_source_git_url = consts::go_source_git_url();
        log::trace!("list upstream go versions via git: {:?}", go_source_git_url);
        let output = Command::new("git")
            .args([
                "ls-remote",
                "--sort=version:refname",
                "--tags",
                &go_source_git_url,
            ])
            .output()?
            .stdout;
        Ok(Regex::new("refs/tags/go(.+)")?
            .captures_iter(&String::from_utf8_lossy(&output))
            .map(|capture| capture[1].to_string())
            .collect())
    }
}
