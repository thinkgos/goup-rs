use std::process::Command;

use clap::Args;
use regex::Regex;

use super::Run;
use crate::pkg::consts;

#[derive(Args, Debug)]
pub struct Search {
    /// a regexp filter
    regex: Option<String>,
}

impl Run for Search {
    fn run(&self) -> Result<(), anyhow::Error> {
        self.list_remote_go_version()?
            .iter()
            .for_each(|v| {
                println!("{}", v);
            });
        Ok(())
    }
}

impl Search {
    fn list_remote_go_version(&self) -> Result<Vec<String>, anyhow::Error> {
        let output = Command::new("git")
            .args([
                "ls-remote",
                "--sort=version:refname",
                "--tags",
                &consts::go_source_git_url(),
            ])
            .output()?
            .stdout;
        let output = String::from_utf8_lossy(&output);

        let re = self.regex.as_deref().filter(|s| !s.is_empty()).map_or_else(
            || "refs/tags/go(.+)".to_owned(),
            |s| format!("refs/tags/go(.*{}.*)", s),
        );
        Ok(Regex::new(&re)?
            .captures_iter(&output)
            .map(|capture| capture[1].to_string())
            .collect())
    }
}
