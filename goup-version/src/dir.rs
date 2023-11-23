use std::{
    fs::File,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
};

use anyhow::anyhow;

/// Dir `${path}/.goup` contain a `PathBuf`.
#[derive(Debug, Clone, PartialEq)]
pub struct Dir {
    path: PathBuf,
}

impl Dir {
    ///  Returns the path to the user's home directory.
    pub fn home_dir() -> Result<PathBuf, anyhow::Error> {
        dirs::home_dir().ok_or_else(|| anyhow!("where is home"))
    }
    /// Allocates a Dir as `${path}/.goup`
    pub fn new<P: AsRef<Path>>(p: P) -> Self {
        let mut path: PathBuf = p.as_ref().into();
        path.push(".goup");
        Self { path }
    }
    /// Allocates a Dir with home_dir as `${home}/.goup`
    pub fn from_home_dir() -> Result<Self, anyhow::Error> {
        let mut path = Self::home_dir()?;
        path.push(".goup");
        Ok(Self { path })
    }
    /// `${path}/.goup/env`
    pub fn env(mut self) -> Self {
        self.path.push("env");
        self
    }
    /// `${path}/.goup/current`
    pub fn current(mut self) -> Self {
        self.path.push("current");
        self
    }
    /// `${path}/.goup/current/bin`
    pub fn current_bin(mut self) -> Self {
        self.path.push("current");
        self.path.push("bin");
        self
    }
    /// `${path}/.goup/bin`
    pub fn bin(mut self) -> Self {
        self.path.push("bin");
        self
    }
    /// `${path}/.goup/{version}`
    pub fn version<P: AsRef<Path>>(mut self, p: P) -> Self {
        self.path.push(p);
        self
    }
    /// `${path}/.goup/{version}/go`
    pub fn version_go<P: AsRef<Path>>(mut self, p: P) -> Self {
        self.path.push(p);
        self.path.push("go");
        self
    }

    // `${path}/.goup/{version}/.unpacked-success` is exist.
    pub fn is_dot_unpacked_success_file_exists<P, P1>(home: P, ver: P1) -> bool
    where
        P: AsRef<Path>,
        P1: AsRef<Path>,
    {
        Self::new(home).version_dot_unpacked_success(ver).exists()
    }
    /// create `${path}/.goup/{version}/.unpacked-success` file
    pub fn create_dot_unpacked_success_file<P, P1>(home: P, ver: P1) -> Result<(), anyhow::Error>
    where
        P: AsRef<Path>,
        P1: AsRef<Path>,
    {
        File::create(Self::new(home).version_dot_unpacked_success(ver))?;
        Ok(())
    }
    /// `${path}/.goup/{version}/.unpacked-success`
    fn version_dot_unpacked_success<P: AsRef<Path>>(mut self, p: P) -> Self {
        self.path.push(p);
        self.path.push(".unpacked-success");
        self
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;

    #[test]
    fn test_dir() {
        let home_dir = Path::new("/home/dev");

        assert_eq!(Dir::new(home_dir).as_ref(), Path::new("/home/dev/.goup"));
        assert_eq!(Dir::new(home_dir).file_name(), Some(OsStr::new(".goup")));

        assert_eq!(
            Dir::new(home_dir).bin().as_ref(),
            Path::new("/home/dev/.goup/bin")
        );
        assert_eq!(
            Dir::new(home_dir).current_bin().as_ref(),
            Path::new("/home/dev/.goup/current/bin")
        );
        assert_eq!(
            Dir::new(home_dir).env().as_ref(),
            Path::new("/home/dev/.goup/env")
        );
        assert_eq!(
            Dir::new(home_dir).current().as_ref(),
            Path::new("/home/dev/.goup/current")
        );
        assert_eq!(
            Dir::new(home_dir).bin().as_ref(),
            Path::new("/home/dev/.goup/bin")
        );
        assert_eq!(
            Dir::new(home_dir).version("go1.21.2").as_ref(),
            Path::new("/home/dev/.goup/go1.21.2")
        );
    }
}
