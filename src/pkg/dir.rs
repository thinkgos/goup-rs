use std::{
    ops::Deref,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq)]
pub struct Dir {
    path: PathBuf,
}

impl Dir {
    pub fn new<P: AsRef<Path>>(p: P) -> Self {
        let mut path: PathBuf = p.as_ref().into();
        path.push(".go");
        Self { path }
    }
    pub fn current_bin(mut self) -> Self {
        self.path.push("current");
        self.path.push("bin");
        self
    }
    pub fn env(mut self) -> Self {
        self.path.push("env");
        self
    }
    pub fn current(mut self) -> Self {
        self.path.push("current");
        self
    }
    pub fn bin(mut self) -> Self {
        self.path.push("bin");
        self
    }
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
        let dir = Dir::new(home_dir);

        assert_eq!(dir.as_ref(), Path::new("/home/dev/.go"));
        assert_eq!(dir.file_name(), Some(OsStr::new(".go")));

        assert_eq!(dir.clone().bin().as_ref(), Path::new("/home/dev/.go/bin"));
        assert_eq!(
            dir.clone().current_bin().as_ref(),
            Path::new("/home/dev/.go/current/bin")
        );
        assert_eq!(dir.clone().env().as_ref(), Path::new("/home/dev/.go/env"));
        assert_eq!(
            dir.clone().current().as_ref(),
            Path::new("/home/dev/.go/current")
        );
        assert_eq!(dir.clone().bin().as_ref(), Path::new("/home/dev/.go/bin"));
    }
}
