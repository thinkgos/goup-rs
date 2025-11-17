#![allow(dead_code)]
use std::sync::LazyLock;
use std::{
    env,
    fmt::{Display, Formatter},
    str::FromStr,
};

pub static SHELL: LazyLock<Option<ShellType>> = LazyLock::new(|| {
    #[cfg(unix)]
    {
        env::var("SHELL")
            .unwrap_or_else(|_| "sh".into())
            .parse()
            .ok()
    }
    #[cfg(windows)]
    {
        env::var("COMSPEC")
            .unwrap_or_else(|_| "powershell".into())
            .parse()
            .ok()
    }
});

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum ShellType {
    Sh,
    Bash,
    Elvish,
    Fish,
    Nu,
    Xonsh,
    Zsh,
    Powershell,
}

impl Display for ShellType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sh => write!(f, "sh"),
            Self::Bash => write!(f, "bash"),
            Self::Elvish => write!(f, "elvish"),
            Self::Fish => write!(f, "fish"),
            Self::Nu => write!(f, "nu"),
            Self::Xonsh => write!(f, "xonsh"),
            Self::Zsh => write!(f, "zsh"),
            Self::Powershell => write!(f, "powershell"),
        }
    }
}

impl FromStr for ShellType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();
        let s = s
            .rsplit_once(['/', '\\'])
            .map(|(_, s)| s.trim_end_matches(".exe"))
            .unwrap_or(&s);

        match s {
            "sh" => Ok(Self::Sh),
            "bash" => Ok(Self::Bash),
            "elvish" => Ok(Self::Elvish),
            "fish" => Ok(Self::Fish),
            "nu" => Ok(Self::Nu),
            "xonsh" => Ok(Self::Xonsh),
            "zsh" => Ok(Self::Zsh),
            "powershell" => Ok(Self::Powershell),
            _ => Err(format!("unsupported shell type: {s}")),
        }
    }
}

impl ShellType {
    pub fn get_or_current(shell: Option<ShellType>) -> Option<ShellType> {
        shell.or(*SHELL)
    }
}

#[cfg(test)]
mod tests {
    use super::ShellType;

    #[test]
    fn test_parse() {
        assert_eq!("sh".parse(), Ok(ShellType::Sh));
        assert_eq!("bash".parse(), Ok(ShellType::Bash));
        assert_eq!("elvish".parse(), Ok(ShellType::Elvish));
        assert_eq!("fish".parse(), Ok(ShellType::Fish));
        assert_eq!("nu".parse(), Ok(ShellType::Nu));
        assert_eq!("xonsh".parse(), Ok(ShellType::Xonsh));
        assert_eq!("zsh".parse(), Ok(ShellType::Zsh));
        assert_eq!("powershell".parse(), Ok(ShellType::Powershell));

        assert_eq!("/bin/zsh".parse(), Ok(ShellType::Zsh));
        assert_eq!(
            "C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe".parse(),
            Ok(ShellType::Powershell)
        );
    }

    #[test]
    fn test_display() {
        assert_eq!(ShellType::Sh.to_string(), "sh");
        assert_eq!(ShellType::Bash.to_string(), "bash");
        assert_eq!(ShellType::Elvish.to_string(), "elvish");
        assert_eq!(ShellType::Fish.to_string(), "fish");
        assert_eq!(ShellType::Nu.to_string(), "nu");
        assert_eq!(ShellType::Xonsh.to_string(), "xonsh");
        assert_eq!(ShellType::Zsh.to_string(), "zsh");
        assert_eq!(ShellType::Powershell.to_string(), "powershell");
    }
}
