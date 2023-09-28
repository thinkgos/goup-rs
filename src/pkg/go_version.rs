use std::fs::DirEntry;

use anyhow::anyhow;
use which::which;

use super::dir::Dir;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct GoVersion {
    // Version: 1.21.1
    pub version: String,
    // active or not
    pub active: bool,
}

impl GoVersion {
    pub fn list() -> Result<Vec<GoVersion>, anyhow::Error> {
        let home = dirs::home_dir().ok_or_else(|| anyhow!(""))?;

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
                Some(GoVersion {
                    version: ver.trim_start_matches("go").into(),
                    active: current == v.path(),
                })
            })
            .collect();
        vers.sort();
        Ok(vers)
    }
}

pub fn show_go_if_exist() {
    if let Ok(go_bin) = which("go") {
        println!(
            "No Go is installed by goup. Using system Go {}.\n",
            go_bin.to_string_lossy()
        );
    } else {
        println!("No Go is installed by goup.")
    }
}
