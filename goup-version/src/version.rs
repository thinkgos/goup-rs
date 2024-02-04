use std::fs;
use std::fs::DirEntry;
use std::ops::Deref;
use std::process::Command;

use anyhow::anyhow;

use regex::Regex;
use reqwest::blocking;

use super::consts;
use super::Dir;
use super::ToolchainFilter;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    // Version: 1.21.1
    pub version: String,
    // active or not
    pub active: bool,
}

impl Version {
    pub fn init_env(s: &str) -> Result<(), anyhow::Error> {
        let goup_dir = Dir::from_home_dir()?;
        if !goup_dir.exists() {
            fs::create_dir_all(&goup_dir)?;
        }
        let env_file = goup_dir.env();
        fs::write(env_file, s)?;
        Ok(())
    }
    #[deprecated(
        since = "0.3.0",
        note = "please use `list_upstream_go_versions` instead"
    )]
    pub fn list_upstream_versions(regex: Option<&str>) -> Result<Vec<String>, anyhow::Error> {
        Self::list_upstream_go_versions(regex.map(|s| ToolchainFilter::Filter(s.to_owned())))
    }
    pub fn list_upstream_go_versions(
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

    pub fn get_upstream_latest_go_version(host: &str) -> Result<String, anyhow::Error> {
        let url = format!("{}/VERSION?m=text", host);
        let body = blocking::get(url)?.text()?;
        let ver = body
            .split('\n')
            .nth(0)
            .ok_or_else(|| anyhow!("Getting latest Go version failed"))?;
        Ok(ver.to_owned())
    }

    pub fn list_go_version() -> Result<Vec<Version>, anyhow::Error> {
        let home = Dir::home_dir()?;
        // may be .goup not exist
        if !Dir::new(&home).exists() {
            return Ok(Vec::new());
        }
        // may be current not exist
        let current = Dir::new(&home).current().read_link();
        let current = current.as_ref();
        let dir: Result<Vec<DirEntry>, _> = Dir::new(&home).read_dir()?.collect();
        let mut version_dirs: Vec<_> = dir?
            .iter()
            .filter_map(|v| {
                if !v.path().is_dir() {
                    return None;
                }

                let ver = v.file_name().to_string_lossy().to_string();
                if ver != "gotip" && !Dir::is_dot_unpacked_success_file_exists(&home, &ver) {
                    return None;
                }
                Some(Version {
                    version: ver.trim_start_matches("go").into(),
                    active: current.is_ok_and(|vv| vv == Dir::new(&home).version_go(ver).deref()),
                })
            })
            .collect();
        version_dirs.sort();
        Ok(version_dirs)
    }

    pub fn set_go_version(version: &str) -> Result<(), anyhow::Error> {
        let version = Self::normalize(version);
        let home = Dir::home_dir()?;
        let original = Dir::new(&home).version_go(&version);
        if !original.exists() {
            return Err(anyhow!(
                "Go version {version} is not installed. Install it with `goup install`."
            ));
        }
        let link = Dir::new(&home).current();
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

    pub fn remove_go_version(version: &str) -> Result<(), anyhow::Error> {
        let version = Self::normalize(version);
        let cur = Self::current_go_version()?;
        if Some(&version) == cur.as_ref() {
            log::warn!("{} is current active version,  ignore deletion!", version);
        } else {
            let version_dir = Dir::from_home_dir()?.version(version);
            if version_dir.exists() {
                fs::remove_dir_all(&version_dir)?;
            }
        }
        Ok(())
    }

    pub fn remove_go_versions(vers: &[&str]) -> Result<(), anyhow::Error> {
        if !vers.is_empty() {
            let home = Dir::home_dir()?;
            let cur = Self::current_go_version()?;
            for ver in vers {
                let version = Self::normalize(ver);
                if Some(&version) == cur.as_ref() {
                    log::warn!("{} is current active version, ignore deletion!", ver);
                    continue;
                }
                let version_dir = Dir::new(&home).version(&version);
                if version_dir.exists() {
                    fs::remove_dir_all(&version_dir)?;
                }
            }
        }
        Ok(())
    }

    pub fn current_go_version() -> Result<Option<String>, anyhow::Error> {
        // may be current not exist
        let current = Dir::from_home_dir()?
            .current()
            .read_link()
            .ok()
            .and_then(|p| {
                p.parent()
                    .and_then(|v| v.file_name().map(|vv| vv.to_string_lossy().to_string()))
            });
        Ok(current)
    }

    pub fn list_dl(contain_sha256: bool) -> Result<Vec<String>, anyhow::Error> {
        let home = Dir::home_dir()?;
        // may be .goup or .goup/dl not exist
        if !Dir::new(&home).exists() || !Dir::new(&home).dl().exists() {
            return Ok(Vec::new());
        }
        let dir: Result<Vec<DirEntry>, _> = Dir::new(&home).dl().read_dir()?.collect();
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
    pub fn remove_dl() -> Result<(), anyhow::Error> {
        let dl_dir = Dir::from_home_dir()?.dl();
        if dl_dir.exists() {
            fs::remove_dir_all(&dl_dir)?;
        }
        Ok(())
    }

    pub fn remove_goup_home() -> Result<(), anyhow::Error> {
        let goup_home_dir = Dir::from_home_dir()?;
        if goup_home_dir.exists() {
            fs::remove_dir_all(&goup_home_dir)?;
        }
        Ok(())
    }

    /// normalize the version string.
    /// 1.21.1   -> go1.21.1
    /// go1.21.1 -> go1.21,1
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
