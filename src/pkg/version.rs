use std::fs::DirEntry;

use super::dir::Dir;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    // Version: 1.21.1
    pub version: String,
    // active or not
    pub active: bool,
}

impl Version {
    pub fn list() -> Result<Vec<Version>, anyhow::Error> {
        let home: std::path::PathBuf = Dir::home_dir()?;
        let current = Dir::new(&home).current().read_link()?;

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
                    active: current == v.path(),
                })
            })
            .collect();
        vers.sort();
        Ok(vers)
    }
}
