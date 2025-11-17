#![allow(dead_code)]
use std::fmt::Display;

use crate::shell::Shell;

pub struct Nushell;

enum EnvOp<'a> {
    Set { key: &'a str, val: &'a str },
    Hide { key: &'a str },
}

impl Display for EnvOp<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EnvOp::Set { key, val } => writeln!(f, "set,{key},{val}"),
            EnvOp::Hide { key } => writeln!(f, "hide,{key},"),
        }
    }
}
impl Nushell {
    fn escape_csv_value(s: &str) -> String {
        if s.contains(['\r', '\n', '"', ',']) {
            format!("\"{}\"", s.replace('"', "\"\""))
        } else {
            s.to_owned()
        }
    }
}
impl Shell for Nushell {
    fn set_env(&self, k: &str, v: &str) -> String {
        let k = Self::escape_csv_value(k);
        let v = Self::escape_csv_value(v);

        EnvOp::Set { key: &k, val: &v }.to_string()
    }

    fn prepend_env(&self, k: &str, v: &str) -> String {
        format!("$env.{k} = ($env.{k} | prepend r#'{v}'#)\n")
    }

    fn unset_env(&self, k: &str) -> String {
        let k = Self::escape_csv_value(k);
        EnvOp::Hide { key: k.as_ref() }.to_string()
    }
}

impl Display for Nushell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "nu")
    }
}
