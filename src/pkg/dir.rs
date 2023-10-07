use std::{
    ops::Deref,
    path::{Path, PathBuf},
};

/// Dir contain a `PathBuf`
#[derive(Debug, Clone, PartialEq)]
pub struct Dir {
    path: PathBuf,
}

impl Dir {
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
