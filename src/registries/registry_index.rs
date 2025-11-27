use std::{process::Command, sync::mpsc, thread, time::Duration};

use anyhow::anyhow;
use regex::Regex;
use reqwest::blocking::Client;
use semver::VersionReq;
use serde::Deserialize;
use serde::Serialize;
use which::which;

use crate::consts;
use crate::registries::local_go_index::{LocalGoIndex, Resolution};
use crate::{toolchain, toolchain::ToolchainFilter};

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
pub struct RegistryIndex {
    host: String,
}

impl RegistryIndex {
    pub fn new(host: &str) -> Self {
        Self {
            host: host.to_owned(),
        }
    }
    /// get upstream latest go version.
    pub fn get_upstream_latest_go_version(&self) -> Result<String, anyhow::Error> {
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

    pub fn match_version_req(&self, version_req: &str) -> Result<String, anyhow::Error> {
        log::debug!("version request: {version_req}");
        let ver_req = VersionReq::parse(version_req)?;

        let search_type = LocalGoIndex::read().map_or(Ok(Resolution::Unresolved), |v| {
            v.try_match_archived_version(&ver_req)
        })?;
        if let Resolution::Resolved(ver) = search_type {
            log::debug!("use archived!!!");
            Ok(ver)
        } else {
            log::debug!("use active!!!");
            self.list_upstream_go_versions_filter(None)?
                .iter()
                .rev()
                .find_map(|v| {
                    toolchain::semantic(v)
                        .ok()
                        .filter(|semver| ver_req.matches(semver))
                        .map(|_| v)
                })
                .map(|v| v.to_owned())
                .ok_or_else(|| anyhow!("no matching version found!"))
        }
    }
    /// list upstream go versions filter by toolchain filter.
    /// NOTE: 此方法每次都尝试更新缓存!
    pub fn list_upstream_go_versions_filter(
        &self,
        filter: Option<&ToolchainFilter>,
    ) -> Result<Vec<String>, anyhow::Error> {
        let ver = self.list_upstream_go_versions()?;
        LocalGoIndex::write_if_change(&ver.clone().into()).ok();
        let Some(filter) = filter else {
            return Ok(ver);
        };
        let re = match filter {
            ToolchainFilter::Stable => {
                r#"(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)(?:\.(?:0|[1-9]\d*))?\b"#.to_string()
            }
            ToolchainFilter::Unstable => {
                r#"(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)(?:\.(?:0|[1-9]\d*))?(?:rc(?:0|[1-9]\d*))"#
                    .to_string()
            }
            ToolchainFilter::Beta => {
                r#"(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)(?:\.(?:0|[1-9]\d*))?(?:beta(?:0|[1-9]\d*))"#
                    .to_string()
            }
            ToolchainFilter::Filter(s) => format!("(.*{s}.*)"),
        };
        let re = Regex::new(&re)?;
        Ok(ver
            .into_iter()
            .filter_map(|v| re.is_match(&v).then_some(v))
            .collect())
    }

    /// list upstream go versions if get go version failure from http then fallback use git.
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

#[cfg(test)]
mod tests {
    use super::LocalGoIndex;

    #[test]
    fn test_cache_go_version_impl_from_vec_trait() {
        {
            let v1 = vec![
                "1.24.0", "1.25.2", "1.24.1", "1.25rc1", "1.25.1", "1.23.2", "1.25.3", "1.24rc1",
                "1.23rc1", "1.23.0", "1.24.2", "1.23.1", "1.25.0",
            ];

            let v2 = v1.iter().map(|s| s.to_string()).collect::<Vec<String>>();
            let cgv: LocalGoIndex = v2.into();
            assert_eq!(cgv.versions, v1);
            assert_eq!(cgv.latest, "1.25.3");
            assert_eq!(cgv.secondary, "1.24.2");
        }
        {
            let v1 = vec![
                "1.24.0",
                "1.24rc1",
                "1.24.2",
                "1.25rc2",
                "1.25beta2",
                "1.23.1",
                "1.25rc1",
                "1.24.1",
                "1.23rc1",
                "1.23.0",
                "1.25beta1",
            ];
            let v2 = v1.iter().map(|s| s.to_string()).collect::<Vec<String>>();
            let cgv: LocalGoIndex = v2.into();
            assert_eq!(cgv.versions, v1);
            assert_eq!(cgv.latest, "1.24.2");
            assert_eq!(cgv.secondary, "1.23.1");
        }
    }
}
