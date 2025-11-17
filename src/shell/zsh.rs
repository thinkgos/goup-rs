#![allow(dead_code)]
use std::fmt::Display;

use crate::shell::{Shell, bash::Bash};

pub struct Zsh;

impl Shell for Zsh {
    fn set_env(&self, k: &str, v: &str) -> String {
        Bash.set_env(k, v)
    }

    fn prepend_env(&self, k: &str, v: &str) -> String {
        format!("export {k}=\"{v}:${k}\"\n")
    }

    fn unset_env(&self, k: &str) -> String {
        Bash.unset_env(k)
    }
}

impl Display for Zsh {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "zsh")
    }
}
