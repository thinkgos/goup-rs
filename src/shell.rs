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
            .unwrap_or_else(|_| "cmd.exe".into())
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
    Pwsh,
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
            Self::Pwsh => write!(f, "pwsh"),
        }
    }
}

impl FromStr for ShellType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();
        let s = s.rsplit_once('/').map(|(_, s)| s).unwrap_or(&s);
        match s {
            "sh" => Ok(Self::Sh),
            "bash" => Ok(Self::Bash),
            "elvish" => Ok(Self::Elvish),
            "fish" => Ok(Self::Fish),
            "nu" => Ok(Self::Nu),
            "xonsh" => Ok(Self::Xonsh),
            "zsh" => Ok(Self::Zsh),
            "pwsh" => Ok(Self::Pwsh),
            _ => Err(format!("unsupported shell type: {s}")),
        }
    }
}

impl ShellType {
    pub fn get_or_current(shell: Option<ShellType>) -> Option<ShellType> {
        shell.or(*SHELL)
    }
}
