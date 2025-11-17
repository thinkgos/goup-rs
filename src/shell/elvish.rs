#![allow(dead_code)]
use std::fmt::Display;

use crate::shell::Shell;

pub struct Elvish;

impl Shell for Elvish {
    fn set_env(&self, k: &str, v: &str) -> String {
        let k = shell_escape::unix::escape(k.into());
        let v = shell_escape::unix::escape(v.into());
        let v = v.replace("\\n", "\n");
        format!("set-env {k} {v}\n")
    }

    fn prepend_env(&self, k: &str, v: &str) -> String {
        let k = shell_escape::unix::escape(k.into());
        let v = shell_escape::unix::escape(v.into());
        format!("set-env {k} {v}(get-env {k})\n")
    }

    fn unset_env(&self, k: &str) -> String {
        format!("unset-env {k}\n", k = shell_escape::unix::escape(k.into()))
    }
}

impl Display for Elvish {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "elvish")
    }
}
