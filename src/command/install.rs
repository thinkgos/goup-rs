use std::{
    env, fs,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use anyhow::anyhow;
use clap::Args;
use flate2::read::GzDecoder;
use reqwest::{blocking, StatusCode};
use sha2::{Digest, Sha256};
use tar::Archive;
// use zip::ZipArchive;

use crate::pkg::{consts, dir::Dir};

use super::{switch_go_version, Run};

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
        switch_go_version(&version)
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
        Self::verify_archive_sha256(
            &archive_file,
            Self::get_archive_sha256(&archive_url)?.trim(),
        )?;
        // 解压
        println!("Unpacking {} ...", archive_file.display());
        Self::unpack_archive(&target_version_dir, &archive_file)?;
        Dir::create_dot_unpacked_success(&home, &version)?;
        // 设置解压成功
        println!(
            "Success: {} installed in {}",
            version,
            target_version_dir.display()
        );
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
    /// 获取压缩包sha256
    fn get_archive_sha256(archive_url: &str) -> Result<String, anyhow::Error> {
        Ok(blocking::get(format!("{}.sha256", archive_url))?.text()?)
    }
    /// 下载压缩包
    fn download_archive(archive_file: &PathBuf, archive_url: &str) -> Result<(), anyhow::Error> {
        let mut response = blocking::get(archive_url)?;
        if !response.status().is_success() {
            return Err(anyhow!("Downloading archive failure"));
        }
        let mut file = File::create(archive_file)?;
        response.copy_to(&mut file)?;
        Ok(())
    }
    /// 校验文件sha256
    fn verify_archive_sha256(
        archive_file: &PathBuf,
        expect_sha256: &str,
    ) -> Result<(), anyhow::Error> {
        let mut context = Sha256::new();
        let mut file = File::open(archive_file)?;
        let mut buffer = [0; 4096]; // 定义一个缓冲区来处理字节流数据
        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            context.update(&buffer[..bytes_read]);
        }
        let result = context.finalize();
        if expect_sha256 != &format!("{:x}", result) {
            return Err(anyhow!(
                "{} corrupt? does not have expected SHA-256 of {}",
                archive_file.display(),
                expect_sha256,
            ));
        }
        Ok(())
    }
    /// unpack_archive unpacks the provided archive zip or tar.gz file to targetDir,
    /// removing the "go/" prefix from file entries.
    fn unpack_archive(
        target_version_dir: &PathBuf,
        archive_file: &PathBuf,
    ) -> Result<(), anyhow::Error> {
        let p = archive_file.to_string_lossy();
        if p.ends_with(".zip") {
            Self::unpack_zip(target_version_dir, archive_file)
        } else if p.ends_with(".tar.gz") {
            Self::unpack_tar_gz(target_version_dir, archive_file)
        } else {
            Err(anyhow!("unsupported archive file"))
        }
    }
    fn unpack_zip(
        _target_version_dir: &PathBuf,
        _archive_file: &PathBuf,
    ) -> Result<(), anyhow::Error> {
        // let zip_file = File::open(archive_file)?;
        // let mut archive = ZipArchive::new(zip_file)?;
        // for i in 0..archive.len() {
        //     let mut file = archive.by_index(i)?;
        //     let out_path = format!("{}/{}", output_dir, file.name());
        //     if file.name().ends_with('/') {
        //         // Create directory if it's a directory entry
        //         std::fs::create_dir_all(&out_path)?;
        //     } else {
        //         // Extract file contents
        //         let mut out_file = File::create(&out_path)?;
        //         io::copy(&mut file, &mut out_file)?;
        //     }
        // }
        Ok(())
    }
    fn unpack_tar_gz(
        target_version_dir: &PathBuf,
        archive_file: &PathBuf,
    ) -> Result<(), anyhow::Error> {
        let tar_gz_file = File::open(archive_file)?;
        let tar = GzDecoder::new(tar_gz_file);
        let mut archive = Archive::new(tar);

        for entry in archive.entries()? {
            let mut entry = entry?;
            let path = entry.path()?;
            let rel = Path::new("go");
            if path.starts_with(rel) {
                let mut dest_file = PathBuf::new();
                dest_file.push(target_version_dir);
                dest_file.push(path.strip_prefix(rel)?); // trim prefix path `go`

                let parent = dest_file.parent().ok_or(anyhow!("No parent path found"))?;
                if !parent.exists() {
                    fs::create_dir_all(parent)?;
                }
                entry.unpack(dest_file)?;
            }
        }
        Ok(())
    }
}
