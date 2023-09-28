use std::process::Command;

use clap::Args;
use regex::Regex;

use super::Run;

#[derive(Args, Debug)]
pub struct Search {
    /// a regexp filter
    regex: Option<String>,
}

impl Run for Search {
    fn run(&self) -> Result<(), anyhow::Error> {
        list_remote_go_version(self.regex.to_owned())?
            .iter()
            .for_each(|v| {
                println!("{}", v);
            });
        Ok(())
    }
}

fn list_remote_go_version(re: Option<String>) -> Result<Vec<String>, anyhow::Error> {
    let output = Command::new("git")
        .args([
            "ls-remote",
            "--sort=version:refname",
            "--tags",
            "https://github.com/golang/go",
        ])
        .output()?
        .stdout;
    let output = String::from_utf8_lossy(&output);

    let re = re.unwrap_or_default();
    let re = format!(
        "refs/tags/go({})",
        if re.is_empty() {
            ".+".to_owned()
        } else {
            format!(r".*{}.*", &re)
        }
    );
    Ok(Regex::new(&re)?
        .captures_iter(&output)
        .map(|capture| capture[1].to_string())
        .collect())
}
