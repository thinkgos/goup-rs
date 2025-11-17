#[cfg(unix)]
use std::os::unix::process::CommandExt;

use std::{
    env,
    path::Path,
    process::{Command, Stdio},
};

use anyhow::anyhow;
use clap::Args;
use dialoguer::{Select, theme::ColorfulTheme};

use crate::{
    shell::ShellType,
    version::{Version, consts::GOUP_GO_VERSION, dir::Dir},
};

use super::Run;

#[derive(Args, Debug, PartialEq)]
#[command(disable_version_flag = true)]
pub struct Shell {
    /// target go version
    version: Option<String>,
    /// custom shell type
    #[arg(short, long)]
    shell: Option<ShellType>,
}

impl Run for Shell {
    fn run(&self) -> Result<(), anyhow::Error> {
        let go_version = if let Some(version) = &self.version {
            Version::normalize(version)
        } else {
            let vers = Version::list_go_version()?;
            if vers.is_empty() {
                return Err(anyhow!(
                    "Not any go is installed, Install it with `goup install`."
                ));
            }
            let mut items = Vec::new();
            let mut pos = 0;
            for (i, v) in vers.iter().enumerate() {
                items.push(v.version.as_ref());
                if v.active {
                    pos = i;
                }
            }
            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select a version")
                .items(&items)
                .default(pos)
                .interact()?;
            Version::normalize(items[selection])
        };
        let goup_home = Dir::goup_home()?;
        if !goup_home.is_dot_unpacked_success_file_exists(&go_version) {
            return Err(anyhow!(
                "Go version {go_version} is not installed, Install it with `goup install`.",
            ));
        }

        let shell =
            ShellType::get_or_current(self.shell).ok_or_else(|| anyhow!("Failed to get shell"))?;
        let env_separator = if cfg!(windows) { ";" } else { ":" };

        let go_root_path = goup_home.version(&go_version);
        let go_root_bin_path = go_root_path.bin();

        let parent_env_go_root = env::var("GOROOT").unwrap_or_default();
        let parent_go_root_bin = (!parent_env_go_root.is_empty()).then(|| {
            Path::new(&parent_env_go_root)
                .join("bin")
                .to_string_lossy()
                .to_string()
        });
        let parent_env_path = env::var("PATH").unwrap_or_default();

        let child_env_go_root = go_root_path.to_string_lossy();
        let child_go_root_bin = go_root_bin_path.to_string_lossy();
        let child_env_path = parent_env_path
            .split(env_separator)
            .filter(|v| {
                if let Some(ref parent_bin) = parent_go_root_bin
                    && *v == parent_bin
                {
                    return false;
                }
                true
            })
            .chain(std::iter::once(child_go_root_bin.as_ref()))
            .collect::<Vec<_>>()
            .join(env_separator);

        log::debug!("Enter new shell with go version: {}", go_version,);
        let mut command = Command::new(shell.to_string());
        let command = command
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .env(GOUP_GO_VERSION, go_version)
            .env("GOROOT", child_env_go_root.as_ref())
            .env("PATH", &child_env_path);
        #[cfg(unix)]
        {
            let err = command.exec();
            Err(err.into())
        }
        #[cfg(windows)]
        {
            let mut child = command.spawn()?;
            let status = child.wait()?;
            log::debug!("Returned to parent shell with exit status: {}", status);

            Ok(())
        }
    }
}
