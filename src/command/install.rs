use std::{
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::anyhow;
use clap::Args;
use reqwest::{blocking, StatusCode};

use crate::pkg::{consts, dir::Dir};

use super::Run;

#[derive(Args, Debug)]
#[command(disable_version_flag = true)]
pub struct Install {
    /// target go version
    version: Option<String>,
    /// an optional change list (CL), If the version is 'tip'
    cl: Option<String>,
    /// host that is used to download Go.
    #[arg(long, default_value_t = consts::GO_HOST.to_owned(), env = "GOUP_GO_HOST")]
    host: String,
}

impl Run for Install {
    fn run(&self) -> Result<(), anyhow::Error> {
        let version = if let Some(s) = self.version.as_deref() {
            if s.starts_with("go") {
                s.to_owned()
            } else {
                format!("go{}", s)
            }
        } else {
            self.get_latest_go_version()?
        };
        if version == "gotip" {
            self.install_go_tip()?;
        } else {
            self.install_go_version(&version)?;
        }
        // switch_go_version(&version)
        Ok(())
    }
}

impl Install {
    fn install_go_tip(&self) -> Result<(), anyhow::Error> {
        //    self.cl.as_deref()
        Ok(())
    }
    fn install_go_version(&self, version: &str) -> Result<(), anyhow::Error> {
        let home = Dir::home_dir()?;
        let target_version_dir = Dir::new(&home).version(&version);

        // 是否已解压并且存在
        if Dir::is_dot_unpacked_success_exists(&home, &version) {
            println!(
                "{}: already installed in {:?}",
                version,
                target_version_dir.display()
            );
            return Ok(());
        }
        // 压缩包url
        let archive_url = consts::go_version_archive_url(version);
        // 压缩包长度
        let archive_content_length = Self::get_archive_content_length(version, &archive_url)?;
        // 压缩包文件名
        let archive_file_name = Path::new(&archive_url)
            .file_name()
            .ok_or_else(|| anyhow!("Getting archive filename failure."))?
            .to_string_lossy();
        let mut archive_file = target_version_dir.clone();
        archive_file.push(archive_file_name.as_ref());

        println!("bingo:\n\t{}\n\t{}", archive_url, archive_file.display());
        if !target_version_dir.exists() {
            fs::create_dir_all(&target_version_dir)?
        }
        if !archive_file.exists() || archive_file.metadata()?.len() != archive_content_length {
            // 下载
            Self::download_archive(&archive_file, &archive_url)?;
            // 检查大小
            let got_archive_content_length = archive_file.metadata()?.len();
            if got_archive_content_length != archive_content_length {
                return Err(anyhow!(
                    "downloaded file {} size {} doesn't match server size {}",
                    archive_file.display(),
                    got_archive_content_length,
                    archive_content_length,
                ));
            }
        }
        // 校验.sha256
        let want_archive_sha256 = Self::get_archive_sha256(&archive_url)?;
        Self::verify_sha256(&archive_file, want_archive_sha256.trim())?;
        // 解压
        Self::unpack_archive(&target_version_dir, &archive_file)?;
        // 设置解压成功

        Ok(())
    }
    fn get_latest_go_version(&self) -> Result<String, anyhow::Error> {
        let url = format!("{}/VERSION?m=text", self.host);
        let body = blocking::get(&url)?.text()?;
        let ver = body
            .split("\n")
            .nth(0)
            .ok_or_else(|| anyhow!("Getting latest Go version failed"))?;
        Ok(ver.to_owned())
    }
    fn get_archive_content_length(version: &str, archive_url: &str) -> Result<u64, anyhow::Error> {
        let resp = blocking::Client::builder()
            .build()?
            .head(archive_url)
            .send()?;
        if resp.status() == StatusCode::NOT_FOUND {
            return Err(anyhow!(
                "no binary release of {} for {}/{} at {}",
                version,
                env::consts::OS,
                env::consts::ARCH,
                archive_url,
            ));
        }
        if !resp.status().is_success() {
            return Err(anyhow!(
                "server returned {} checking size of {}",
                resp.status().canonical_reason().unwrap_or_default(),
                archive_url,
            ));
        }
        let content_length = if let Some(header_value) = resp.headers().get("Content-Length") {
            header_value.to_str()?.parse()?
        } else {
            0
        };
        Ok(content_length)
    }
    fn get_archive_sha256(archive_url: &str) -> Result<String, anyhow::Error> {
        let body = blocking::get(format!("{}.sha256", archive_url))?.text()?;
        Ok(body)
    }
    fn download_archive(_archive_file: &PathBuf, _archive_url: &str) -> Result<(), anyhow::Error> {
        Ok(())
    }
    fn verify_sha256(_archive_file: &PathBuf, want_sha256: &str) -> Result<(), anyhow::Error> {
        Ok(())
    }
    fn unpack_archive(
        _target_version_dir: &PathBuf,
        _archive_file: &PathBuf,
    ) -> Result<(), anyhow::Error> {
        Ok(())
    }
}
