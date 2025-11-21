#![allow(dead_code)]
use std::{
    env,
    fs::{self, File},
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
};

use anyhow::anyhow;

use crate::consts::GOUP_HOME;

/// Dir `${path}/.goup` contain a `PathBuf`.
#[derive(Debug, Clone, PartialEq)]
pub struct Dir {
    path: PathBuf,
}

impl Dir {
    /// Returns the path to the user's home directory.
    pub fn home_dir() -> Result<PathBuf, anyhow::Error> {
        dirs::home_dir().ok_or_else(|| anyhow!("home dir get failed"))
    }
    /// Allocates a Dir as `${path}/.goup`
    pub fn new<P: AsRef<Path>>(p: P) -> Self {
        let mut path: PathBuf = p.as_ref().into();
        path.push(".goup");
        Self { path }
    }
    /// Allocates a `GOUP_HOME` Dir as Environment Or `${HOME}/.goup`
    pub fn goup_home() -> Result<Self, anyhow::Error> {
        env::var(GOUP_HOME)
            .ok()
            .filter(|s| !s.is_empty())
            .map(|s| {
                Ok(Self {
                    path: PathBuf::from(s),
                })
            })
            .unwrap_or_else(|| Self::home_dir().map(Self::new))
    }
    // Creates an owned [`Dir`] with `path` adjoined to `self`.
    pub fn join_path<P: AsRef<Path>>(&self, path: P) -> Self {
        Self {
            path: self.path.join(path),
        }
    }
    /// Extends `self` with name
    pub fn env(&self, name: &str) -> Self {
        self.join_path(name)
    }
    /// Extends `self` with `current`.
    pub fn current(&self) -> Self {
        self.join_path("current")
    }
    /// Extends `self` with `/current/bin`
    pub fn current_bin(&self) -> Self {
        let mut d = self.join_path("current");
        d.push("bin");
        d
    }
    /// Extends `self` with `/bin`
    pub fn bin(&self) -> Self {
        self.join_path("bin")
    }
    /// Extends `self` with `{version}`
    pub fn version<P: AsRef<Path>>(&self, ver: P) -> Self {
        self.join_path(ver)
    }
    /// Extends `self` with `cache`
    pub fn cache(&self) -> Self {
        self.join_path("cache")
    }
    /// Extends `self` with `cache/{filename}`
    pub fn cache_file<P: AsRef<Path>>(&self, p: P) -> Self {
        let mut d = self.join_path("cache");
        d.push(p);
        d
    }
    /// Extends `self` with `{version}/.unpacked-success`
    fn version_dot_unpacked_success<P: AsRef<Path>>(&self, ver: P) -> Self {
        let mut d = self.join_path(ver);
        d.push(".unpacked-success");
        d
    }
    /// `${path}/.goup/{version}/.unpacked-success` is exist.
    pub fn is_dot_unpacked_success_file_exists<P>(&self, ver: P) -> bool
    where
        P: AsRef<Path>,
    {
        self.version_dot_unpacked_success(&ver).exists()
    }
    /// create `${path}/.goup/{version}/.unpacked-success` file
    pub fn create_dot_unpacked_success_file<P>(&self, ver: P) -> Result<(), anyhow::Error>
    where
        P: AsRef<Path>,
    {
        let dot_unpacked_success_file = self.version_dot_unpacked_success(&ver);
        let parent = dot_unpacked_success_file.parent();
        if let Some(parent) = parent {
            fs::create_dir_all(parent)?;
        }
        File::create(&dot_unpacked_success_file)?;
        Ok(())
    }
    /// Extends `self` with `index-go.json`
    pub fn index_go(&self) -> Self {
        self.join_path("index-go.json")
    }
}

impl AsRef<Path> for Dir {
    fn as_ref(&self) -> &Path {
        &self.path
    }
}

impl Deref for Dir {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.path
    }
}

impl DerefMut for Dir {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.path
    }
}

impl Default for Dir {
    fn default() -> Self {
        Self {
            path: PathBuf::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;

    #[test]
    fn test_home_dir() {
        println!("Dir - home_dir: {:?}", Dir::home_dir());
        println!("Dir - from_home_dir: {:?}", Dir::goup_home());
    }
    #[test]
    fn test_dir() {
        let home_dir = Path::new("/home/dev");

        assert_eq!(Dir::new(home_dir).as_ref(), Path::new("/home/dev/.goup"));
        assert_eq!(Dir::new(home_dir).file_name(), Some(OsStr::new(".goup")));

        assert_eq!(
            Dir::new(home_dir).env("env").as_ref(),
            Path::new("/home/dev/.goup/env")
        );
        assert_eq!(
            Dir::new(home_dir).current().as_ref(),
            Path::new("/home/dev/.goup/current")
        );
        assert_eq!(
            Dir::new(home_dir).current_bin().as_ref(),
            Path::new("/home/dev/.goup/current/bin")
        );
        assert_eq!(
            Dir::new(home_dir).bin().as_ref(),
            Path::new("/home/dev/.goup/bin")
        );
        assert_eq!(
            Dir::new(home_dir).cache().as_ref(),
            Path::new("/home/dev/.goup/cache")
        );
        assert_eq!(
            Dir::new(home_dir).cache_file("file").as_ref(),
            Path::new("/home/dev/.goup/cache/file")
        );
        assert_eq!(
            Dir::new(home_dir).version("go1.21.2").as_ref(),
            Path::new("/home/dev/.goup/go1.21.2")
        );
    }

    #[test]
    fn test_dot_unpacked_success_file() -> Result<(), anyhow::Error> {
        let tmp_home_dir = tempfile::tempdir()?;
        let tmp_goup_home = Dir::new(tmp_home_dir);
        println!("{}", tmp_goup_home.display());
        assert!(!tmp_goup_home.is_dot_unpacked_success_file_exists("go1.21.2"));
        tmp_goup_home.create_dot_unpacked_success_file("go1.21.2")?;
        assert!(tmp_goup_home.is_dot_unpacked_success_file_exists("go1.21.2"));
        Ok(())
    }
}
