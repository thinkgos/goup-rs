use std::fs;
use std::fs::DirEntry;
use std::ops::Deref;
use std::process::Command;

use anyhow::anyhow;
use goup_consts::consts;
use regex::Regex;
use reqwest::blocking;

use super::Dir;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    // Version: 1.21.1
    pub version: String,
    // active or not
    pub active: bool,
}

impl Version {
    pub fn init_env(s: &str) -> Result<(), anyhow::Error> {
        let env_file = Dir::from_home_dir()?.env();
        fs::write(env_file, s)?;
        Ok(())
    }
    pub fn list_upstream_versions(regex: Option<&str>) -> Result<Vec<String>, anyhow::Error> {
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

        let re = regex.filter(|s| !s.is_empty()).map_or_else(
            || "refs/tags/go(.+)".to_owned(),
            |s| format!("refs/tags/go(.*{}.*)", s),
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
        // may be goup not exist
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
                if ver == "gotip" || !ver.starts_with("go") {
                    return None;
                }
                if !Dir::is_dot_unpacked_success_file_exists(&home, &ver) {
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
        if !Dir::is_dot_unpacked_success_file_exists(&home, &version) {
            return Err(anyhow!(
                "Go version {version} is not installed. Install it with `goup install`."
            ));
        }
        let source_dir = Dir::new(&home).version_go(&version);
        let current = Dir::new(&home).current();
        let _ = fs::remove_dir_all(&current);
        #[cfg(unix)]
        {
            use std::os::unix::fs as unix_fs;
            unix_fs::symlink(source_dir, &current)?;
        }
        #[cfg(windows)]
        {
            junction::create(source_dir, &current)?;
        }
        println!("Default Go is set to '{version}'");
        Ok(())
    }

    pub fn remove_go_version(version: &str) -> Result<(), anyhow::Error> {
        let version = Self::normalize(version);
        let version_dir = Dir::from_home_dir()?.version(version);
        if version_dir.exists() {
            fs::remove_dir_all(&version_dir)?;
        }
        Ok(())
    }

    pub fn remove_go_versions(vers: &[&str]) -> Result<(), anyhow::Error> {
        let home = Dir::home_dir()?;
        for ver in vers {
            let version = Self::normalize(ver);
            let version_dir = Dir::new(&home).version(&version);
            if version_dir.exists() {
                fs::remove_dir_all(&version_dir)?;
            }
        }
        Ok(())
    }
    /// normalize the version string.
    /// 1.21.1   -> go1.21.1
    /// go1.21.1 -> go1.21,1
    pub fn normalize(version: &str) -> String {
        if version.starts_with("go") {
            version.to_string()
        } else {
            format!("go{}", version)
        }
    }
}
