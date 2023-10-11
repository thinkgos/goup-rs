use std::{
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
};

use anyhow::anyhow;

/// Dir contain a `PathBuf`
#[derive(Debug, Clone, PartialEq)]
pub struct Dir {
    path: PathBuf,
}

impl Dir {
    /// current home dir
    pub fn home_dir() -> Result<PathBuf, anyhow::Error> {
        dirs::home_dir().ok_or_else(|| anyhow!("where is home"))
    }
    /// Allocates a Dir with home_dir as `${home}/.go`
    pub fn new_with_home_dir() -> Result<Self, anyhow::Error> {
        let mut path = Self::home_dir()?;
        path.push(".go");
        Ok(Self { path })
    }
    /// Allocates a Dir as `${path}/.go`
    pub fn new<P: AsRef<Path>>(p: P) -> Self {
        let mut path: PathBuf = p.as_ref().into();
        path.push(".go");
        Self { path }
    }
    /// `${path}/.go/current/bin`
    pub fn current_bin(mut self) -> Self {
        self.path.push("current");
        self.path.push("bin");
        self
    }
    /// `${path}/.go/env`
    pub fn env(mut self) -> Self {
        self.path.push("env");
        self
    }
    /// `${path}/.go/current`
    pub fn current(mut self) -> Self {
        self.path.push("current");
        self
    }
    /// `${path}/.go/bin`
    pub fn bin(mut self) -> Self {
        self.path.push("bin");
        self
    }
    /// `${path}/.go/{version}`
    pub fn version<P: AsRef<Path>>(mut self, p: P) -> Self {
        self.path.push(p);
        self
    }
    /// `${path}/.go/{version}/.unpacked-success`
    fn version_dot_unpacked_success<P: AsRef<Path>>(mut self, p: P) -> Self {
        self.path.push(p);
        self.path.push(".unpacked-success");
        self
    }
    pub fn is_dot_unpacked_success_exists<P: AsRef<Path>, P1: AsRef<Path>>(
        home: P,
        ver: P1,
    ) -> bool {
        Self::new(home).version_dot_unpacked_success(ver).exists()
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

        assert_eq!(Dir::new(home_dir).as_ref(), Path::new("/home/dev/.go"));
        assert_eq!(Dir::new(home_dir).file_name(), Some(OsStr::new(".go")));

        assert_eq!(
            Dir::new(home_dir).bin().as_ref(),
            Path::new("/home/dev/.go/bin")
        );
        assert_eq!(
            Dir::new(home_dir).current_bin().as_ref(),
            Path::new("/home/dev/.go/current/bin")
        );
        assert_eq!(
            Dir::new(home_dir).env().as_ref(),
            Path::new("/home/dev/.go/env")
        );
        assert_eq!(
            Dir::new(home_dir).current().as_ref(),
            Path::new("/home/dev/.go/current")
        );
        assert_eq!(
            Dir::new(home_dir).bin().as_ref(),
            Path::new("/home/dev/.go/bin")
        );
        assert_eq!(
            Dir::new(home_dir).version("go1.21.2").as_ref(),
            Path::new("/home/dev/.go/go1.21.2")
        );
    }
}
