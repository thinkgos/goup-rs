use std::fs;
use std::fs::DirEntry;
use std::ops::Deref;
use std::process::Command;

use anyhow::anyhow;
use anyhow::Ok;
use regex::Regex;
use reqwest::blocking;
use serde::{Deserialize, Serialize};

use super::consts;
use super::Dir;
use super::ToolchainFilter;

#[derive(Serialize, Deserialize, Debug)]
pub struct GoFile {
    pub arch: String,
    pub filename: String,
    pub kind: String,
    pub os: String,
    pub sha256: String,
    pub size: isize,
    pub version: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct GoRelease {
    pub version: String,
    pub stable: bool,
    // pub files: Vec<GoFile>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    // Version: 1.21.1
    pub version: String,
    // active or not
    pub active: bool,
}

impl Version {
    /// initializes the environment file.
    pub fn init_env(s: &str) -> Result<(), anyhow::Error> {
        let goup_home = Dir::goup_home()?;
        if !goup_home.exists() {
            fs::create_dir_all(&goup_home)?;
        }
        let env_file = goup_home.env();
        fs::write(env_file, s)?;
        Ok(())
    }
    /// list upstream go version, use http.
    #[deprecated(
        since = "0.7.3",
        note = "use `list_upstream_go_versions_from_git` or `list_upstream_go_versions_from_http`  instead"
    )]
    pub fn list_upstream_go_versions(
        filter: Option<ToolchainFilter>,
    ) -> Result<Vec<String>, anyhow::Error> {
        Self::list_upstream_go_versions_from_git(filter)
    }

    /// list upstream go version, use git.
    pub fn list_upstream_go_versions_from_git(
        filter: Option<ToolchainFilter>,
    ) -> Result<Vec<String>, anyhow::Error> {
        let output = Command::new("git")
            .args([
                "ls-remote",
                "--sort=version:refname",
                "--tags",
                &consts::go_source_git_url(),
            ])
            .output()?
            .stdout;
        let output = String::from_utf8_lossy(&output);

        let re = filter.map_or_else(
            || "refs/tags/go(.+)".to_owned(),
            |f| match f {
                ToolchainFilter::Stable => {
                    r#"refs/tags/go((?:0|[1-9]\d*)\.(?:0|[1-9]\d*)(?:\.(?:0|[1-9]\d*))?\b)"#
                        .to_string()
                }
                ToolchainFilter::Unstable => {
                    r#"refs/tags/go((?:0|[1-9]\d*)\.(?:0|[1-9]\d*)(?:\.(?:0|[1-9]\d*))?(?:rc(?:0|[1-9]\d*)))"#
                        .to_string()
                }
                ToolchainFilter::Beta => {
                    r#"refs/tags/go((?:0|[1-9]\d*)\.(?:0|[1-9]\d*)(?:\.(?:0|[1-9]\d*))?(?:beta(?:0|[1-9]\d*)))"#
                        .to_string()
                }
                ToolchainFilter::Filter(s) => format!("refs/tags/go(.*{}.*)", s),
            },
        );
        Ok(Regex::new(&re)?
            .captures_iter(&output)
            .map(|capture| capture[1].to_string())
            .collect())
    }
    /// list upstream go version, use http.
    pub fn list_upstream_go_versions_from_http(
        host: &str,
        filter: Option<ToolchainFilter>,
    ) -> Result<Vec<String>, anyhow::Error> {
        let url = format!("{}/dl/?mode=json&include=all", host);
        let go_releases: Vec<GoRelease> = blocking::get(url)?.json()?;

        let re = filter.map_or_else(
            || "go(.+)".to_owned(),
            |f| match f {
                ToolchainFilter::Stable => {
                    r#"go((?:0|[1-9]\d*)\.(?:0|[1-9]\d*)(?:\.(?:0|[1-9]\d*))?\b)"#
                        .to_string()
                }
                ToolchainFilter::Unstable => {
                    r#"go((?:0|[1-9]\d*)\.(?:0|[1-9]\d*)(?:\.(?:0|[1-9]\d*))?(?:rc(?:0|[1-9]\d*)))"#
                        .to_string()
                }
                ToolchainFilter::Beta => {
                    r#"go((?:0|[1-9]\d*)\.(?:0|[1-9]\d*)(?:\.(?:0|[1-9]\d*))?(?:beta(?:0|[1-9]\d*)))"#
                        .to_string()
                }
                ToolchainFilter::Filter(s) => format!("go(.*{}.*)", s),
            },
        );
        let re = Regex::new(&re)?;
        Ok(go_releases
            .into_iter()
            .filter_map(|v| {
                if re.is_match(&v.version) {
                    Some(v.version.trim_start_matches("go").to_string())
                } else {
                    None
                }
            })
            .rev()
            .collect())
    }

    /// get upstream latest go version.
    pub fn get_upstream_latest_go_version(host: &str) -> Result<String, anyhow::Error> {
        let url = format!("{}/VERSION?m=text", host);
        let body = blocking::get(url)?.text()?;
        let ver = body
            .split('\n')
            .nth(0)
            .ok_or_else(|| anyhow!("Getting latest Go version failed"))?;
        Ok(ver.to_owned())
    }
    /// list locally installed go version.
    pub fn list_go_version() -> Result<Vec<Version>, anyhow::Error> {
        let goup_home = Dir::goup_home()?;
        // may be .goup not exist
        if !goup_home.exists() {
            return Ok(Vec::new());
        }

        // may be current not exist
        let current = goup_home.current().read_link();
        let current = current.as_ref();
        let dir: Result<Vec<DirEntry>, _> = goup_home.read_dir()?.collect();
        let mut version_dirs: Vec<_> = dir?
            .iter()
            .filter_map(|v| {
                if !v.path().is_dir() {
                    return None;
                }

                let ver = v.file_name().to_string_lossy().to_string();
                if ver != "gotip" && !goup_home.is_dot_unpacked_success_file_exists(&ver) {
                    return None;
                }
                Some(Version {
                    version: ver.trim_start_matches("go").into(),
                    active: current.is_ok_and(|vv| vv == goup_home.version_go(ver).deref()),
                })
            })
            .collect();
        version_dirs.sort();
        Ok(version_dirs)
    }

    /// set active go version
    pub fn set_go_version(version: &str) -> Result<(), anyhow::Error> {
        let version = Self::normalize(version);
        let goup_home = Dir::goup_home()?;
        let original = goup_home.version_go(&version);
        if !original.exists() {
            return Err(anyhow!(
                "Go version {version} is not installed. Install it with `goup install`."
            ));
        }
        let link = goup_home.current();
        let _ = fs::remove_dir_all(&link);
        #[cfg(unix)]
        {
            use std::os::unix::fs as unix_fs;
            unix_fs::symlink(original, &link)?;
        }
        #[cfg(windows)]
        {
            junction::create(original, &link)?;
        }
        log::info!("Default Go is set to '{version}'");
        Ok(())
    }
    /// remove the go version, if it is current active go version, will ignore deletion.
    pub fn remove_go_version(version: &str) -> Result<(), anyhow::Error> {
        let version = Self::normalize(version);
        let cur = Self::current_go_version()?;
        if Some(&version) == cur.as_ref() {
            log::warn!("{} is current active version,  ignore deletion!", version);
        } else {
            let version_dir = Dir::goup_home()?.version(version);
            if version_dir.exists() {
                fs::remove_dir_all(&version_dir)?;
            }
        }
        Ok(())
    }

    /// remove multiple go version, if it is current active go version, will ignore deletion.
    pub fn remove_go_versions(vers: &[&str]) -> Result<(), anyhow::Error> {
        if !vers.is_empty() {
            let goup_home = Dir::goup_home()?;
            let cur = Self::current_go_version()?;
            for ver in vers {
                let version = Self::normalize(ver);
                if Some(&version) == cur.as_ref() {
                    log::warn!("{} is current active version, ignore deletion!", ver);
                    continue;
                }
                let version_dir = goup_home.version(&version);
                if version_dir.exists() {
                    fs::remove_dir_all(&version_dir)?;
                }
            }
        }
        Ok(())
    }

    /// current active go version
    pub fn current_go_version() -> Result<Option<String>, anyhow::Error> {
        // may be current not exist
        let current = Dir::goup_home()?.current().read_link().ok().and_then(|p| {
            p.parent()
                .and_then(|v| v.file_name().map(|vv| vv.to_string_lossy().to_string()))
        });
        Ok(current)
    }

    /// list `${HOME}/.goup/dl` directory items(only file, ignore directory).
    pub fn list_dl(contain_sha256: Option<bool>) -> Result<Vec<String>, anyhow::Error> {
        let goup_home = Dir::goup_home()?;
        // may be .goup or .goup/dl not exist
        if !goup_home.exists() || !goup_home.dl().exists() {
            return Ok(Vec::new());
        }
        let contain_sha256 = contain_sha256.unwrap_or_default();
        let dir: Result<Vec<DirEntry>, _> = goup_home.dl().read_dir()?.collect();
        let mut archive_files: Vec<_> = dir?
            .iter()
            .filter_map(|v| {
                if v.path().is_dir() {
                    return None;
                }
                let filename = v.file_name();
                let filename = filename.to_string_lossy();
                (contain_sha256 || !filename.ends_with(".sha256")).then(|| filename.to_string())
            })
            .collect();
        archive_files.sort();
        Ok(archive_files)
    }

    /// remove `${HOME}/.goup/dl` directory.
    pub fn remove_dl() -> Result<(), anyhow::Error> {
        let dl_dir = Dir::goup_home()?.dl();
        if dl_dir.exists() {
            fs::remove_dir_all(&dl_dir)?;
        }
        Ok(())
    }

    /// remove `${HOME}/.goup` directory.
    pub fn remove_goup_home() -> Result<(), anyhow::Error> {
        let goup_home_dir = Dir::goup_home()?;
        if goup_home_dir.exists() {
            fs::remove_dir_all(&goup_home_dir)?;
        }
        Ok(())
    }

    /// normalize the version string.
    /// 1.21.1   -> go1.21.1
    /// go1.21.1 -> go1.21.1
    /// tip      -> gotip
    /// gotip    -> gotip
    pub fn normalize(ver: &str) -> String {
        if ver.starts_with("go") {
            ver.to_string()
        } else {
            format!("go{}", ver)
        }
    }
}
