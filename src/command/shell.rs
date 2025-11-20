#[cfg(unix)]
use std::os::unix::process::CommandExt;

use std::{
    env, fs,
    path::Path,
    process::{Command, Stdio},
};

use anyhow::anyhow;
use clap::Args;
use dialoguer::{Select, theme::ColorfulTheme};

use crate::{
    command::utils::InstallOptions,
    consts::GOUP_GO_VERSION,
    dir::Dir,
    registry::{Registry, RegistryIndex},
    shell::ShellType,
    toolchain,
    version::Version,
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
    #[command(flatten)]
    install_options: InstallOptions,
}

impl Run for Shell {
    fn run(&self) -> Result<(), anyhow::Error> {
        let local_versions = Version::list_go_version()?;
        let go_version = self.get_target_version(&local_versions)?;

        let mut current_shell_version = None;
        let mut current_default_version = None;
        for v in local_versions.iter() {
            if v.session {
                current_shell_version = Some(&v.version);
            }
            if v.default {
                current_default_version = Some(&v.version);
            }
        }

        if current_shell_version == Some(&go_version)
            || current_shell_version.is_none() && current_default_version == Some(&go_version)
        {
            // 如果当前会话和目标一样, 则不切换
            // 不在会话当中, 如果默认和目标一样, 则不切换
            log::info!(
                "Current environment already uses Go {go_version}, skip enter new shell session.",
            );

            return Ok(());
        }

        let go_version = toolchain::normalize(&go_version);
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

        log::info!("Enter new shell session with Go {go_version}");
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

impl Shell {
    fn get_target_version(&self, local_versions: &[Version]) -> Result<String, anyhow::Error> {
        let target_version = if let Some(version) = &self.version {
            if !local_versions.iter().any(|v| v.version == *version) {
                let registry = Registry::new(
                    &self.install_options.registry,
                    self.install_options.skip_verify,
                    self.install_options.enable_check_archive_size,
                );
                registry.install_go(&toolchain::normalize(version))?
            }
            version.to_owned()
        } else if let Some(ver) = self.get_mod_file_version(local_versions) {
            ver
        } else {
            if local_versions.is_empty() {
                return Err(anyhow!(
                    "Not any go is installed, Install it with `goup install`."
                ));
            }
            let mut items = Vec::new();

            let mut session_pos = None;
            let mut default_pos = None;
            for (i, v) in local_versions.iter().enumerate() {
                items.push(&v.version);
                if v.session {
                    session_pos = Some(i);
                }
                if v.default {
                    default_pos = Some(i);
                }
            }
            let pos = session_pos.unwrap_or(default_pos.unwrap_or(0));
            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select a version")
                .items(&items)
                .default(pos)
                .interact()?;
            items[selection].to_owned()
        };

        Ok(target_version)
    }

    fn get_mod_file_version(&self, local_versions: &[Version]) -> Option<String> {
        let current_dir = env::current_dir().ok()?;
        let mod_go_version = ["go.work", "go.mod"]
            .into_iter()
            .find_map(|filename| Self::parse_go_mod_or_work_file(current_dir.join(filename)))?;

        let version_req = match mod_go_version.chars().filter(|&v| v == '.').count() {
            0 | 1 => format!("~{mod_go_version}"),
            _ => format!("={mod_go_version}"),
        };
        let version = RegistryIndex::new(&self.install_options.registry_index)
            .match_version_req(&version_req)
            .ok()?;
        if !local_versions.iter().any(|v| v.version == version) {
            let registry = Registry::new(
                &self.install_options.registry,
                self.install_options.skip_verify,
                self.install_options.enable_check_archive_size,
            );
            registry.install_go(&toolchain::normalize(&version)).ok();
        }
        Some(version)
    }
    fn parse_go_mod_or_work_file(path: impl AsRef<Path>) -> Option<String> {
        if !path.as_ref().exists() {
            return None;
        }
        let mut target = None;
        let content = fs::read_to_string(path).ok()?;
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("//") || line.is_empty() {
                continue;
            }
            // https://go.dev/ref/mod#go-mod-file-go
            // go directive
            if let Some(rest) = line.strip_prefix("go ") {
                // go 1.22
                let v = rest.trim().to_string();
                target = Some(v);
                continue;
            }
            // https://go.dev/ref/mod#go-mod-file-toolchain
            // toolchain directive
            if let Some(rest) = line.strip_prefix("toolchain ") {
                // toolchain go1.22.3
                let rest = rest.trim().trim_start_matches("go").to_string();
                if rest.is_empty() {
                    continue;
                }
                target = Some(rest);
                continue;
            }
        }
        target
    }
}
