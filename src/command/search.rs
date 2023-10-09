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
        list_remote_go_version(self.regex.as_ref())?
            .iter()
            .for_each(|v| {
                println!("{}", v);
            });
        Ok(())
    }
}

fn list_remote_go_version(re: Option<&String>) -> Result<Vec<String>, anyhow::Error> {
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

    let re = re.filter(|s| !s.is_empty()).map_or_else(
        || "refs/tags/go(.+)".to_owned(),
        |s| format!("refs/tags/go(.*{}.*)", s),
    );
    Ok(Regex::new(&re)?
        .captures_iter(&output)
        .map(|capture| capture[1].to_string())
        .collect())
}
