use std::fs;
use std::fs::DirEntry;

use anyhow::Result;
use anyhow::anyhow;

use crate::consts;
use crate::dir::Dir;
use crate::toolchain;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    // Version: 1.21.1
    pub version: String,
    // default or not
    pub default: bool,
    // session or not.
    pub session: bool,
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

    /// list locally installed go version.
    pub fn list_go_version() -> Result<Vec<Version>, anyhow::Error> {
        let goup_home = Dir::goup_home()?;
        // may be .goup not exist
        if !goup_home.exists() {
            return Ok(Vec::new());
        }
        // may be active not exist
        let default = goup_home.current().read_link().ok();
        let session = consts::go_version()
            .map(|ver| goup_home.version(ver).to_path_buf())
            .filter(|p| p.exists());

        let dir: Result<Vec<DirEntry>, _> = goup_home.read_dir()?.collect();
        let mut version_dirs: Vec<_> = dir?
            .iter()
            .filter_map(|v| {
                v.path()
                    .is_dir()
                    .then(|| {
                        let ver = v.file_name();
                        let ver = ver.to_string_lossy();
                        (ver.starts_with("go")
                            && (ver == "gotip"
                                || goup_home.is_dot_unpacked_success_file_exists(ver.as_ref())))
                        .then(|| {
                            let vvx = goup_home.version(ver.as_ref());
                            Version {
                                version: ver.trim_start_matches("go").into(),
                                default: default.as_ref().is_some_and(|vv| vv == vvx.as_ref()),
                                session: session.as_ref().is_some_and(|vv| vv == vvx.as_ref()),
                            }
                        })
                    })
                    .flatten()
            })
            .collect();
        version_dirs.sort();
        Ok(version_dirs)
    }

    /// set active go version
    pub fn set_go_version(version: &str) -> Result<(), anyhow::Error> {
        let version = toolchain::normalize(version);
        let goup_home = Dir::goup_home()?;
        let original = goup_home.version(&version);
        if !original.exists() || !goup_home.is_dot_unpacked_success_file_exists(&version) {
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
        let version = toolchain::normalize(version);
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
                let version = toolchain::normalize(ver);
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
}
