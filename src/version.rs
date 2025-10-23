pub mod consts;
pub mod dir;
pub mod toolchain;

use std::fs;
use std::fs::DirEntry;
use std::ops::Deref;
use std::process::Command;
use std::time::Duration;

use anyhow::Result;
use anyhow::anyhow;
use regex::Regex;
use reqwest::blocking::Client;
use semver::Op;
use semver::Version as SemVersion;
use semver::VersionReq;
use serde::{Deserialize, Serialize};
use which::which;

// use consts;
use dir::Dir;
use toolchain::ToolchainFilter;

const HTTP_TIMEOUT: Duration = Duration::from_secs(10);

#[allow(dead_code)]
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
    #[allow(dead_code)]
    pub fn init_env(s: &str) -> Result<(), anyhow::Error> {
        let goup_home = Dir::goup_home()?;
        if !goup_home.exists() {
            fs::create_dir_all(&goup_home)?;
        }
        let env_file = goup_home.env();
        fs::write(env_file, s)?;
        Ok(())
    }
    pub fn list_upstream_go_versions_filter(
        host: &str,
        filter: Option<ToolchainFilter>,
    ) -> Result<Vec<String>, anyhow::Error> {
        let ver = Self::list_upstream_go_versions(host)?;
        let re = filter.map_or_else(
            || "(.+)".to_owned(),
            |f| match f {
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
            },
        );
        let re = Regex::new(&re)?;
        Ok(ver
            .into_iter()
            .filter_map(|v| re.is_match(&v).then_some(v))
            .collect())
    }

    /// list upstream go versions if get go version failure from http then fallback use git.
    pub fn list_upstream_go_versions(host: &str) -> Result<Vec<String>, anyhow::Error> {
        Self::list_upstream_go_versions_from_http(host).or_else(|e| {
            which("git").map_or_else(|_| Err(e), |_| Self::list_upstream_go_versions_from_git())
        })
    }
    /// list upstream go versions from http.
    fn list_upstream_go_versions_from_http(host: &str) -> Result<Vec<String>, anyhow::Error> {
        Ok(Client::builder()
            .timeout(HTTP_TIMEOUT)
            .build()?
            .get(format!("{host}/dl/?mode=json&include=all"))
            .send()?
            .json::<Vec<GoRelease>>()?
            .into_iter()
            .map(|v| v.version.trim_start_matches("go").to_string())
            .rev()
            .collect())
    }
    /// list upstream go versions from git.
    fn list_upstream_go_versions_from_git() -> Result<Vec<String>, anyhow::Error> {
        let output = Command::new("git")
            .args([
                "ls-remote",
                "--sort=version:refname",
                "--tags",
                &consts::go_source_git_url(),
            ])
            .output()?
            .stdout;
        Ok(Regex::new("refs/tags/go(.+)")?
            .captures_iter(&String::from_utf8_lossy(&output))
            .map(|capture| capture[1].to_string())
            .collect())
    }
    pub fn match_version_req(host: &str, ver_pattern: &str) -> Result<String, anyhow::Error> {
        log::debug!("version request pattern: {ver_pattern}");
        let ver_req = VersionReq::parse(ver_pattern)?;
        // 是否是精确匹配, 如果是则直接返回
        if ver_req.comparators.iter().all(|v| v.op == Op::Exact) {
            return Ok(ver_pattern.trim_start_matches('=').to_owned());
        }
        for ver in Self::list_upstream_go_versions(host)?.iter().rev() {
            if ver_req.matches(&Self::semantic(ver)?) {
                return Ok(ver.to_owned());
            }
        }
        Err(anyhow!("not any match version!"))
    }

    /// get upstream latest go version.
    pub fn get_upstream_latest_go_version(host: &str) -> Result<String, anyhow::Error> {
        let body = Client::builder()
            .timeout(HTTP_TIMEOUT)
            .build()?
            .get(format!("{host}/VERSION?m=text"))
            .send()?
            .text()?;
        body.split('\n')
            .nth(0)
            .ok_or_else(|| anyhow!("Getting latest Go version failed"))
            .map(|v| v.to_owned())
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
                if ver == "current"
                    || ver != "gotip" && !goup_home.is_dot_unpacked_success_file_exists(&ver)
                {
                    return None;
                }
                Some(Version {
                    version: ver.trim_start_matches("go").into(),
                    active: current.is_ok_and(|vv| vv == goup_home.version(ver).deref()),
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
        let original = goup_home.version(&version);
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
    #[allow(dead_code)]
    pub fn remove_go_version(version: &str) -> Result<(), anyhow::Error> {
        let version = Self::normalize(version);
        let cur = Self::current_go_version()?;
        if Some(&version) == cur.as_ref() {
            log::warn!("{version} is current active version,  ignore deletion!");
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
                    log::warn!("{ver} is current active version, ignore deletion!");
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
        let current = Dir::goup_home()?
            .current()
            .read_link()
            .ok()
            .and_then(|p| p.file_name().map(|vv| vv.to_string_lossy().to_string()));
        Ok(current)
    }

    /// list `${HOME}/.goup/cache` directory items(only file, ignore directory).
    pub fn list_cache(contain_sha256: Option<bool>) -> Result<Vec<String>, anyhow::Error> {
        let goup_home = Dir::goup_home()?;
        // may be .goup or .goup/cache not exist
        if !goup_home.exists() || !goup_home.cache().exists() {
            return Ok(Vec::new());
        }
        let contain_sha256 = contain_sha256.unwrap_or_default();
        let dir: Result<Vec<DirEntry>, _> = goup_home.cache().read_dir()?.collect();
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

    /// remove `${HOME}/.goup/cache` directory.
    pub fn remove_cache() -> Result<(), anyhow::Error> {
        let dl_dir = Dir::goup_home()?.cache();
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
            format!("go{ver}")
        }
    }
    /// semantic go version string.
    /// 1           -> 1.0.0
    /// 1.21        -> 1.21.0
    /// 1.21rc2     -> 1.21.0-rc2
    /// 1.21.1rc2   -> 1.21.1-rc2
    /// 1.21-rc2    -> 1.21.0-rc2
    /// 1.21.1-rc2  -> 1.21.1-rc2
    /// 1.21.1      -> 1.21.1
    pub fn semantic(ver: &str) -> Result<SemVersion> {
        let count_dot = |name: &str| name.chars().filter(|&v| v == '.').count();
        let name = ver
            .find("alpha")
            .or_else(|| ver.find("beta"))
            .or_else(|| ver.find("rc"))
            .map_or_else(
                || match count_dot(ver) {
                    0 => format!("{ver}.0.0"),
                    1 => format!("{ver}.0"),
                    _ => ver.to_string(),
                },
                |idx| {
                    let start = &ver[..idx].trim_end_matches('-');
                    if count_dot(start) == 2 {
                        format!("{}-{}", start, &ver[idx..])
                    } else {
                        format!("{}.0-{}", start, &ver[idx..])
                    }
                },
            );
        Ok(SemVersion::parse(&name)?)
    }
}

#[cfg(test)]
mod tests {
    use super::Version;
    use semver::Version as SemVersion;

    #[test]
    fn test_normalize() {
        assert_eq!(Version::normalize("1.21.1"), "go1.21.1",);
        assert_eq!(Version::normalize("go1.21.1"), "go1.21.1",);
        assert_eq!(Version::normalize("tip"), "gotip",);
        assert_eq!(Version::normalize("gotip"), "gotip",);
    }

    #[test]
    fn test_semantic() {
        assert_eq!(
            Version::semantic("1").unwrap(),
            "1.0.0".parse::<SemVersion>().unwrap(),
        );
        assert_eq!(
            Version::semantic("1.21").unwrap(),
            "1.21.0".parse::<SemVersion>().unwrap(),
        );
        assert_eq!(
            Version::semantic("1.21rc2").unwrap(),
            "1.21.0-rc2".parse::<SemVersion>().unwrap(),
        );
        assert_eq!(
            Version::semantic("1.21.1rc2").unwrap(),
            "1.21.1-rc2".parse::<SemVersion>().unwrap(),
        );
        assert_eq!(
            Version::semantic("1.21-rc2").unwrap(),
            "1.21.0-rc2".parse::<SemVersion>().unwrap(),
        );
        assert_eq!(
            Version::semantic("1.21.1-rc2").unwrap(),
            "1.21.1-rc2".parse::<SemVersion>().unwrap(),
        );
        assert_eq!(
            Version::semantic("1.21.1").unwrap(),
            "1.21.1".parse::<SemVersion>().unwrap(),
        );
    }

    #[test]
    fn test_all_go_version_semantic() {
        let go_versions = [
            "1",
            "1.2.2",
            "1.3rc1",
            "1.3rc2",
            "1.3",
            "1.3.1",
            "1.3.2",
            "1.3.3",
            "1.4beta1",
            "1.4rc1",
            "1.4rc2",
            "1.4",
            "1.4.1",
            "1.4.2",
            "1.4.3",
            "1.5beta1",
            "1.5beta2",
            "1.5beta3",
            "1.5rc1",
            "1.5",
            "1.5.1",
            "1.5.2",
            "1.5.3",
            "1.5.4",
            "1.6beta1",
            "1.6beta2",
            "1.6rc1",
            "1.6rc2",
            "1.6",
            "1.6.1",
            "1.6.2",
            "1.6.3",
            "1.6.4",
            "1.7beta1",
            "1.7beta2",
            "1.7rc1",
            "1.7rc2",
            "1.7rc3",
            "1.7rc4",
            "1.7rc5",
            "1.7rc6",
            "1.7",
            "1.7.1",
            "1.7.3",
            "1.7.4",
            "1.7.5",
            "1.7.6",
            "1.8beta1",
            "1.8beta2",
            "1.8rc1",
            "1.8rc2",
            "1.8rc3",
            "1.8",
            "1.8.1",
            "1.8.2",
            "1.8.3",
            "1.8.4",
            "1.8.5",
            "1.8.6",
            "1.8.7",
            "1.9beta1",
            "1.9beta2",
            "1.9rc1",
            "1.9rc2",
            "1.9",
            "1.9.1",
            "1.9.2rc2",
            "1.9.2",
            "1.9.3",
            "1.9.4",
            "1.9.5",
            "1.9.6",
            "1.9.7",
            "1.10beta1",
            "1.10beta2",
            "1.10rc1",
            "1.10rc2",
            "1.10",
            "1.10.1",
            "1.10.2",
            "1.10.3",
            "1.10.4",
            "1.10.5",
            "1.10.6",
            "1.10.7",
            "1.10.8",
            "1.11beta1",
            "1.11beta2",
            "1.11beta3",
            "1.11rc1",
            "1.11rc2",
            "1.11",
            "1.11.1",
            "1.11.2",
            "1.11.3",
            "1.11.4",
            "1.11.5",
            "1.11.6",
            "1.11.7",
            "1.11.8",
            "1.11.9",
            "1.11.10",
            "1.11.11",
            "1.11.12",
            "1.11.13",
            "1.12beta1",
            "1.12beta2",
            "1.12rc1",
            "1.12",
            "1.12.1",
            "1.12.2",
            "1.12.3",
            "1.12.4",
            "1.12.5",
            "1.12.6",
            "1.12.7",
            "1.12.8",
            "1.12.9",
            "1.12.10",
            "1.12.11",
            "1.12.12",
            "1.12.13",
            "1.12.14",
            "1.12.15",
            "1.12.16",
            "1.12.17",
            "1.13beta1",
            "1.13rc1",
            "1.13rc2",
            "1.13",
            "1.13.1",
            "1.13.2",
            "1.13.3",
            "1.13.4",
            "1.13.5",
            "1.13.6",
            "1.13.7",
            "1.13.8",
            "1.13.9",
            "1.13.10",
            "1.13.11",
            "1.13.12",
            "1.13.13",
            "1.13.14",
            "1.13.15",
            "1.14beta1",
            "1.14rc1",
            "1.14",
            "1.14.1",
            "1.14.2",
            "1.14.3",
            "1.14.4",
            "1.14.5",
            "1.14.6",
            "1.14.7",
            "1.14.8",
            "1.14.9",
            "1.14.10",
            "1.14.11",
            "1.14.12",
            "1.14.13",
            "1.14.14",
            "1.14.15",
            "1.15beta1",
            "1.15rc1",
            "1.15rc2",
            "1.15",
            "1.15.1",
            "1.15.2",
            "1.15.3",
            "1.15.4",
            "1.15.5",
            "1.15.6",
            "1.15.7",
            "1.15.8",
            "1.15.9",
            "1.15.10",
            "1.15.11",
            "1.15.12",
            "1.15.13",
            "1.15.14",
            "1.15.15",
            "1.16beta1",
            "1.16rc1",
            "1.16",
            "1.16.1",
            "1.16.2",
            "1.16.3",
            "1.16.4",
            "1.16.5",
            "1.16.6",
            "1.16.7",
            "1.16.8",
            "1.16.9",
            "1.16.10",
            "1.16.11",
            "1.16.12",
            "1.16.13",
            "1.16.14",
            "1.16.15",
            "1.17beta1",
            "1.17rc1",
            "1.17rc2",
            "1.17",
            "1.17.1",
            "1.17.2",
            "1.17.3",
            "1.17.4",
            "1.17.5",
            "1.17.6",
            "1.17.7",
            "1.17.8",
            "1.17.9",
            "1.17.10",
            "1.17.11",
            "1.17.12",
            "1.17.13",
            "1.18beta1",
            "1.18beta2",
            "1.18rc1",
            "1.18",
            "1.18.1",
            "1.18.2",
            "1.18.3",
            "1.18.4",
            "1.18.5",
            "1.18.6",
            "1.18.7",
            "1.18.8",
            "1.18.9",
            "1.18.10",
            "1.19beta1",
            "1.19rc1",
            "1.19rc2",
            "1.19",
            "1.19.1",
            "1.19.2",
            "1.19.3",
            "1.19.4",
            "1.19.5",
            "1.19.6",
            "1.19.7",
            "1.19.8",
            "1.19.9",
            "1.19.10",
            "1.19.11",
            "1.19.12",
            "1.19.13",
            "1.20rc1",
            "1.20rc2",
            "1.20rc3",
            "1.20",
            "1.20.1",
            "1.20.2",
            "1.20.3",
            "1.20.4",
            "1.20.5",
            "1.20.6",
            "1.20.7",
            "1.20.8",
            "1.20.9",
            "1.20.10",
            "1.20.11",
            "1.20.12",
            "1.20.13",
            "1.20.14",
            "1.21rc2",
            "1.21rc3",
            "1.21rc4",
            "1.21.0",
            "1.21.1",
            "1.21.2",
            "1.21.3",
            "1.21.4",
            "1.21.5",
            "1.21.6",
            "1.21.7",
            "1.21.8",
            "1.21.9",
            "1.21.10",
            "1.21.12",
            "1.21.13",
            "1.22rc1",
            "1.22rc2",
            "1.22.0",
            "1.22.1",
            "1.22.2",
            "1.22.3",
            "1.22.4",
            "1.22.5",
            "1.22.6",
            "1.22.7",
            "1.22.8",
            "1.22.9",
            "1.22.10",
            "1.22.11",
            "1.23rc1",
            "1.23rc2",
            "1.23.0",
            "1.23.1",
            "1.23.2",
            "1.23.3",
            "1.23.4",
            "1.23.5",
            "1.23.6",
            "1.24rc1",
            "1.24rc2",
            "1.24rc3",
            "1.24.0",
            "1.24.1",
            "1.24.2",
            "1.24.3",
            "1.24.4",
            "1.25rc1",
            "1.25rc2",
            "1.25.0",
            "1.25.1",
            "1.25.2",
            "1.25.3",
        ];
        for ver in go_versions {
            assert!(Version::semantic(ver).is_ok())
        }
    }
}
