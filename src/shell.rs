#![allow(dead_code)]
use std::sync::LazyLock;
use std::{
    env,
    fmt::{Display, Formatter},
    str::FromStr,
};

mod bash;
mod elvish;
mod fish;
mod nushell;
mod powershell;
mod xonsh;
mod zsh;

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
    Bash,
    Elvish,
    Fish,
    NuShell,
    Xonsh,
    Zsh,
    PowerShell,
}

impl ShellType {
    pub fn as_shell(&self) -> Box<dyn Shell> {
        match self {
            Self::Bash => Box::new(bash::Bash),
            Self::Elvish => Box::new(elvish::Elvish),
            Self::Fish => Box::new(fish::Fish),
            Self::NuShell => Box::new(nushell::Nushell),
            Self::Xonsh => Box::new(xonsh::Xonsh),
            Self::Zsh => Box::new(zsh::Zsh),
            Self::PowerShell => Box::new(powershell::PowerShell),
        }
    }
}
impl Display for ShellType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bash => write!(f, "bash"),
            Self::Elvish => write!(f, "elvish"),
            Self::Fish => write!(f, "fish"),
            Self::NuShell => write!(f, "nu"),
            Self::Xonsh => write!(f, "xonsh"),
            Self::Zsh => write!(f, "zsh"),
            Self::PowerShell => write!(f, "pwsh"),
        }
    }
}

impl FromStr for ShellType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();
        let s = s.rsplit_once('/').map(|(_, s)| s).unwrap_or(&s);
        match s {
            "bash" | "sh" => Ok(Self::Bash),
            "elvish" => Ok(Self::Elvish),
            "fish" => Ok(Self::Fish),
            "nu" => Ok(Self::NuShell),
            "xonsh" => Ok(Self::Xonsh),
            "zsh" => Ok(Self::Zsh),
            "pwsh" => Ok(Self::PowerShell),
            _ => Err(format!("unsupported shell type: {s}")),
        }
    }
}

pub trait Shell: Display {
    fn set_env(&self, k: &str, v: &str) -> String;
    fn prepend_env(&self, k: &str, v: &str) -> String;
    fn unset_env(&self, k: &str) -> String;
}

pub fn get_shell(shell: Option<ShellType>) -> Option<Box<dyn Shell>> {
    shell.or(*SHELL).map(|st| st.as_shell())
}
