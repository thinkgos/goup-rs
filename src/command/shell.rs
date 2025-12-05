#[cfg(unix)]
use std::os::unix::process::CommandExt;

use std::{
    collections::{HashMap, HashSet},
    env, fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use anyhow::anyhow;
use clap::Args;
use dialoguer::{Select, theme::ColorfulTheme};
use semver::VersionReq;

use crate::{
    command::utils::{InstallOptions, KeyValuePair},
    consts::GOUP_GO_VERSION,
    dir::Dir,
    registries::{go_index::GoIndex, registry::Registry},
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
    /// skip autodetect go.work/go.mod.
    #[arg(long)]
    skip_autodetect: bool,
    /// custom env vars
    #[arg(short, long)]
    envs: Vec<KeyValuePair>,
    /// env file path
    #[arg(short, long)]
    filename: Option<PathBuf>,
    #[command(flatten)]
    install_options: InstallOptions,
}

impl Run for Shell {
    fn run(&self) -> Result<(), anyhow::Error> {
        let filter_keys = ["GOROOT", GOUP_GO_VERSION];

        let mut envs = if let Some(ref filename) = self.filename {
            dotenvy::from_filename_iter(filename)?
                .filter_map(|v| {
                    v.ok().and_then(|(k, v)| {
                        // 过滤 GOROOT 和 GOUP_GO_VERSION
                        (!filter_keys.contains(&k.as_str())).then_some((k, v))
                    })
                })
                .collect()
        } else {
            HashMap::new()
        };
        // 合并命令行的 -e KEY=VALUE
        for kv in &self.envs {
            if !filter_keys.contains(&kv.key.as_str()) {
                envs.insert(kv.key.clone(), kv.value.clone());
            }
        }

        let local_versions = Version::list_go_version()?;
        let target_go_version = self.get_target_version(&local_versions)?;
        let target_go_version = toolchain::normalize(&target_go_version);
        let goup_home = Dir::goup_home()?;
        if !goup_home.is_dot_unpacked_success_file_exists(&target_go_version) {
            return Err(anyhow!(
                "Go version {target_go_version} is not installed, Install it with `goup install`.",
            ));
        }

        let shell =
            ShellType::get_or_current(self.shell).ok_or_else(|| anyhow!("Failed to get shell"))?;
        let env_separator = if cfg!(windows) { ";" } else { ":" };

        let go_root_path = goup_home.version(&target_go_version);
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
        let child_env_path = {
            let mut seen = HashSet::new();
            envs.get("path")
                .map(|v| v.split(env_separator))
                .into_iter()
                .flatten()
                .chain(std::iter::once(child_go_root_bin.as_ref()))
                .chain(parent_env_path.split(env_separator).filter(|v| {
                    if let Some(parent_bin) = parent_go_root_bin.as_deref()
                        && *v == parent_bin
                    {
                        return false;
                    }
                    true
                }))
                .filter(|v| seen.insert(*v)) // 去重，保持顺序
                .collect::<Vec<_>>()
                .join(env_separator)
        };

        log::info!("Enter new shell session with Go {target_go_version}");
        let mut command = Command::new(shell.to_string());
        let command = command
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .env(GOUP_GO_VERSION, target_go_version)
            .env("GOROOT", child_env_go_root.as_ref())
            .env("PATH", &child_env_path)
            .envs(envs.into_iter().filter(|v| v.0 != "path"));
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
        if let Some(version) = &self.version {
            // 指定了版本号，直接使用该版本
            if !local_versions.iter().any(|v| v.version == *version) {
                let registry = Registry::new(
                    &self.install_options.registry,
                    self.install_options.skip_verify,
                    self.install_options.enable_check_archive_size,
                );
                registry.install_go(&toolchain::normalize(version))?
            }
            return Ok(version.to_owned());
        }
        if !self.skip_autodetect
            && let Some(ver) = self.get_mod_file_version(local_versions)
        {
            // 自动从 go.work/go.mod 文件中获取到版本号
            return Ok(ver);
        }
        // 交互式选择版本号
        if local_versions.is_empty() {
            return Err(anyhow!(
                "Not any go is installed, Install it with `goup install`."
            ));
        }
        let mut items = Vec::new();

        let mut pos = None;
        for (i, v) in local_versions.iter().enumerate() {
            items.push(&v.version);
            if v.session {
                pos = Some(i);
            }
            if v.default {
                pos = pos.or(Some(i));
            }
        }
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select a version")
            .items(&items)
            .default(pos.unwrap_or_default())
            .interact()?;
        Ok(items[selection].to_owned())
    }

    fn get_mod_file_version(&self, local_versions: &[Version]) -> Option<String> {
        let current_dir = env::current_dir().ok()?;
        // 优先获取go.work, 其次go.mod
        let mod_go_version = ["go.work", "go.mod"]
            .into_iter()
            .find_map(|filename| Self::parse_go_mod_or_work_file(current_dir.join(filename)))?;

        let version_req = match mod_go_version.chars().filter(|&v| v == '.').count() {
            0 | 1 => format!("~{mod_go_version}"),
            _ => format!("={mod_go_version}"),
        };
        // let version = RegistryIndex::new(&self.install_options.registry_index)
        //     .match_version_req(&version_req)
        //     .ok()?;
        // 从本地索引中匹配版本号
        let ver_req = VersionReq::parse(&version_req).ok()?;
        let version = GoIndex::read().and_then(|v| v.match_version(&ver_req))?;
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
        let mut version = None;
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
                let rest = rest.trim().to_string();
                if !rest.is_empty() {
                    version = version.or(Some(rest));
                }
                continue;
            }
            // https://go.dev/ref/mod#go-mod-file-toolchain
            // toolchain directive
            if let Some(rest) = line.strip_prefix("toolchain ") {
                // toolchain go1.22.3
                let rest = rest.trim().trim_start_matches("go").to_string();
                if !rest.is_empty() {
                    version = Some(rest);
                    break;
                }
            }
        }
        version
    }
}
