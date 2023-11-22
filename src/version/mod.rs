use std::fs;
use std::fs::DirEntry;

use anyhow::anyhow;
use reqwest::blocking;

use super::dir::Dir;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    // Version: 1.21.1
    pub version: String,
    // active or not
    pub active: bool,
}

impl Version {
    pub fn list_local_version() -> Result<Vec<Version>, anyhow::Error> {
        let home = Dir::home_dir()?;
        // may be current not exist
        let current = Dir::new(&home).current().read_link();
        let current = current.as_ref();
        let dir: Result<Vec<DirEntry>, _> = Dir::new(&home).read_dir()?.collect();
        let mut vers: Vec<_> = dir?
            .iter()
            .filter_map(|v| {
                if !v.path().is_dir() {
                    return None;
                }

                let ver: String = v.file_name().to_string_lossy().to_string();
                if ver == "gotip" || !ver.starts_with("go") {
                    return None;
                }
                if !Dir::is_dot_unpacked_success_exists(&home, &ver) {
                    return None;
                }

                Some(Version {
                    version: ver.trim_start_matches("go").into(),
                    active: current.is_ok_and(|vv| vv == &v.path()),
                })
            })
            .collect();
        vers.sort();
        Ok(vers)
    }
    pub fn get_latest_go_version(host: &str) -> Result<String, anyhow::Error> {
        let url = format!("{}/VERSION?m=text", host);
        let body = blocking::get(url)?.text()?;
        let ver = body
            .split('\n')
            .nth(0)
            .ok_or_else(|| anyhow!("Getting latest Go version failed"))?;
        Ok(ver.to_owned())
    }

    pub fn use_go_version(version: &str) -> Result<(), anyhow::Error> {
        let version = if version.starts_with("go") {
            version.to_string()
        } else {
            format!("go{}", version)
        };
        let home: std::path::PathBuf = Dir::home_dir()?;
        if !Dir::is_dot_unpacked_success_exists(&home, &version) {
            return Err(anyhow!(
                "Go version {version} is not installed. Install it with `govm install`."
            ));
        }
        let source_dir = Dir::new(&home).version(&version);
        let current = Dir::new(&home).current();
        fs::remove_dir_all(&current)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs as unix_fs;
            unix_fs::symlink(source_dir, &current)?;
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs as windows_fs;
            windows_fs::symlink_dir(source_dir, &current)?;
        }
        println!("Default Go is set to '{version}'");
        Ok(())
    }
}
