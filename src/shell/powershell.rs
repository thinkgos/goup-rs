#![allow(dead_code)]
use std::{borrow::Cow, fmt::Display};

use crate::shell::Shell;

pub struct PowerShell;

impl PowerShell {
    fn escape(s: Cow<str>) -> Cow<str> {
        let needs_escape = s.is_empty();

        if !needs_escape {
            return s;
        }

        let mut es = String::with_capacity(s.len());
        let mut chars = s.chars().peekable();
        loop {
            match chars.next() {
                Some('\t') => {
                    es.push_str("`t");
                }
                Some('\n') => {
                    es.push_str("`n");
                }
                Some('\r') => {
                    es.push_str("`r");
                }
                Some('\'') => {
                    es.push_str("`'");
                }
                Some('`') => {
                    es.push_str("``");
                }
                Some(c) => {
                    es.push(c);
                }
                None => {
                    break;
                }
            }
        }
        es.into()
    }
}

impl Shell for PowerShell {
    fn set_env(&self, k: &str, v: &str) -> String {
        let k = Self::escape(k.into());
        let v = Self::escape(v.into());
        format!("$Env:{k}='{v}'\n")
    }

    fn prepend_env(&self, k: &str, v: &str) -> String {
        let k = Self::escape(k.into());
        let v = Self::escape(v.into());
        format!("$Env:{k}='{v}'+[IO.Path]::PathSeparator+$env:{k}\n")
    }

    fn unset_env(&self, k: &str) -> String {
        let k = Self::escape(k.into());
        format!("Remove-Item -ErrorAction SilentlyContinue -Path Env:/{k}\n")
    }
}

impl Display for PowerShell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "pwsh")
    }
}
