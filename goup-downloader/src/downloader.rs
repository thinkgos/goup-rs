use std::{
    env, fs,
    fs::File,
    io::{self, Read},
    path::{Path, PathBuf},
};

use anyhow::anyhow;
use flate2::read::GzDecoder;
use reqwest::{blocking, StatusCode};
use sha2::{Digest, Sha256};
use tar::Archive;
use zip::ZipArchive;

use goup_consts::consts;
use goup_version::Dir;

pub struct Downloader;

impl Downloader {
    pub fn install_go_tip(_cl: Option<&str>) -> Result<(), anyhow::Error> {
        Err(anyhow!("Feature not supported"))
    }
    pub fn install_go_version(version: &str) -> Result<(), anyhow::Error> {
        let home = Dir::home_dir()?;
        let dest_version_dir = Dir::new(&home).version(version);

        // 是否已解压并且存在
        if Dir::is_dot_unpacked_success_file_exists(&home, version) {
            println!(
                "{}: already installed in {:?}",
                version,
                dest_version_dir.display()
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
        let mut archive_file = dest_version_dir.clone();
        archive_file.push(archive_file_name.as_ref());

        if !dest_version_dir.exists() {
            fs::create_dir_all(&dest_version_dir)?
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
        Self::unpack_archive(&dest_version_dir, &archive_file)?;
        Dir::create_dot_unpacked_success_file(&home, version)?;
        // 设置解压成功
        println!(
            "Success: {} installed in {}",
            version,
            dest_version_dir.display()
        );
        Ok(())
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
        let got_sha256 = format!("{:x}", result);
        if expect_sha256 != got_sha256 {
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
    fn unpack_archive(dest_dir: &Path, archive_file: &PathBuf) -> Result<(), anyhow::Error> {
        let p = archive_file.to_string_lossy();
        if p.ends_with(".zip") {
            Self::unpack_zip(dest_dir, archive_file)
        } else if p.ends_with(".tar.gz") {
            Self::unpack_tgz(dest_dir, archive_file)
        } else {
            Err(anyhow!("unsupported archive file"))
        }
    }
    /// unpack_zip unpack *.zip
    fn unpack_zip(dest_dir: &Path, archive_file: &Path) -> Result<(), anyhow::Error> {
        let mut archive = ZipArchive::new(File::open(archive_file)?)?;
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let path = file.mangled_name();

            let dest_file = dest_dir.join(path);
            let parent = dest_file.parent().ok_or(anyhow!("No parent path found"))?;
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }

            let mut output_file = File::create(dest_file)?;
            io::copy(&mut file, &mut output_file)?;
        }
        Ok(())
    }

    /// unpack_tgz unpack *.tar.gz
    fn unpack_tgz(dest_dir: &Path, archive_file: &PathBuf) -> Result<(), anyhow::Error> {
        let mut archive = Archive::new(GzDecoder::new(File::open(archive_file)?));
        for entry in archive.entries()? {
            let mut entry = entry?;
            let path = entry.path()?;

            let dest_file = dest_dir.join(path);
            let parent = dest_file.parent().ok_or(anyhow!("No parent path found"))?;
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
            entry.unpack(dest_file)?;
        }
        Ok(())
    }
}
