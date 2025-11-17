#![allow(dead_code)]
use std::fmt::Display;

use itertools::Itertools;

use crate::shell::Shell;

pub struct Fish;

impl Shell for Fish {
    fn set_env(&self, key: &str, v: &str) -> String {
        let k = shell_escape::unix::escape(key.into());
        // Fish uses space-separated list for PATH, not colon-separated string
        if key == "PATH" {
            let paths = v
                .split(':')
                .map(|p| shell_escape::unix::escape(p.into()))
                .join(" ");
            format!("set -gx PATH {paths}\n")
        } else {
            let v = shell_escape::unix::escape(v.into());
            format!("set -gx {k} {v}\n")
        }
    }

    fn prepend_env(&self, key: &str, value: &str) -> String {
        let k = shell_escape::unix::escape(key.into());

        // match key {
        //     env_key if env_key == *env::PATH_KEY => env::split_paths(value)
        //         .filter_map(|path| {
        //             let path_str = path.to_str()?;
        //             if path_str.is_empty() {
        //                 None
        //             } else {
        //                 Some(format!(
        //                     "fish_add_path --global --path {}\n",
        //                     shell_escape::unix::escape(path_str.into())
        //                 ))
        //             }
        //         })
        //         .collect::<String>(),
        //     _ => {
        let v = shell_escape::unix::escape(value.into());
        format!("set -gx {k} {v} ${k}\n")
        // }
        // }
    }

    fn unset_env(&self, k: &str) -> String {
        format!("set -e {k}\n", k = shell_escape::unix::escape(k.into()))
    }
}

impl Display for Fish {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "fish")
    }
}
