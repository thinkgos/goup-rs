use std::{
    env,
    ffi::OsStr,
    fs,
    fs::File,
    io::{BufRead, BufReader, Read},
    path::Path,
    process::{Command, Stdio},
};

use anyhow::anyhow;
use reqwest::{blocking, StatusCode};
use sha2::{Digest, Sha256};

use goup_version::consts;
use goup_version::Dir;

use crate::archived::Unpack;

pub struct Downloader;

impl Downloader {
    fn execute_command<P, I, S>(program: S, working_dir: P, args: I) -> Result<(), anyhow::Error>
    where
        P: AsRef<Path>,
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let mut command = Command::new(program)
            .current_dir(working_dir)
            .args(args)
            .stdout(Stdio::piped())
            .spawn()?;
        if let Some(stdout) = command.stdout.take() {
            let reader = BufReader::new(stdout);

            for line in reader.lines().flatten() {
                println!("{}", line);
            }
        }
        let status = command.wait()?;
        if !status.success() {
            return Err(anyhow!("Command failed with {}", status));
        }
        Ok(())
    }

    pub fn install_go_tip(_cl: Option<&str>) -> Result<(), anyhow::Error> {
        let gotip_go = Dir::from_home_dir()?.version_go("gotip");
        let gotip_git = gotip_go.join(".git");
        // gotip is not clone from source
        if !gotip_git.exists() {
            fs::create_dir_all(&gotip_go)?;
            //* git clone --depth=1 {url}
            Self::execute_command(
                "git",
                &gotip_go,
                [
                    "clone",
                    "--depth=1",
                    &consts::go_source_git_url(),
                    &gotip_go.to_string_lossy(),
                ],
            )?;
            //* git remote add upstream {url}
            Self::execute_command(
                "git",
                &gotip_go,
                [
                    "remote",
                    "add",
                    "upstream",
                    &consts::go_source_upstream_git_url(),
                ],
            )?;
        }
        log::info!("Updating the go development tree...");
        //* git fetch origin master
        Self::execute_command("git", &gotip_go, ["fetch", "origin", "master"])?;
        //* git -c advice.detachedHead=false checkout FETCH_HEAD
        Self::execute_command(
            "git",
            &gotip_go,
            ["-c", "advice.detachedHead=false", "checkout", "FETCH_HEAD"],
        )?;
        //* git clean -i -d
        Self::execute_command("git", &gotip_go, ["clean", "-i", "-d"])?;
        //* git clean -q -f -d -X
        Self::execute_command("git", &gotip_go, ["clean", "-q", "-f", "-d", "-X"])?;
        // 执行 ./src/make.rs/make.bat/make.bash 有$GOROOT/src问题

        //* $HOME/{owner}/.goup/gotip/src/<make.bash|make.rs|make.bat>

        let script = match env::consts::OS {
            "windows" => "make.bat",
            "plan9" => "make.rs",
            _ => "make.bash",
        };

        Self::execute_command(gotip_go.join("src").join(script), gotip_go.join("src"), [])?;
        Ok(())
    }
    pub fn install_go_version(version: &str) -> Result<(), anyhow::Error> {
        let home = Dir::home_dir()?;
        let version_dest_dir = Dir::new(&home).version(version);
        // 是否已解压成功并且存在
        if Dir::is_dot_unpacked_success_file_exists(&home, version) {
            log::info!(
                "{}: already installed in {:?}",
                version,
                version_dest_dir.display()
            );
            return Ok(());
        }
        // download directory
        let dl_dest_dir = Dir::new(&home).dl();
        // 压缩包文件名称
        let archive_filename = consts::go_version_archive(version);
        // 压缩包sha256文件名称
        let archive_sha256_filename = consts::archive_sha256(&archive_filename);
        // 压缩包url
        let (archive_url, archive_sha256_url) = consts::archive_url(&archive_filename);
        if !dl_dest_dir.exists() {
            log::debug!("Create download directory");
            fs::create_dir_all(&dl_dest_dir)?
        }

        // 压缩包文件
        let archive_file = dl_dest_dir.join(archive_filename);
        let archive_sha256_file = dl_dest_dir.join(archive_sha256_filename);
        if !archive_file.exists()
            || !archive_sha256_file.exists()
            || Self::verify_archive_file_sha256(&archive_file, &archive_sha256_file).is_err()
        {
            log::debug!(
                "Download archive file from {} to {}",
                archive_url,
                archive_file.display(),
            );
            // 下载压缩包
            Self::download_archive(&archive_file, &archive_url)?;
            log::debug!("Check archive file content length");
            // 压缩包长度
            let archive_content_length =
                Self::get_upstream_archive_content_length(version, &archive_url)?;
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
            // 下载压缩包sha256
            log::debug!(
                "Download archive sha256 file from {} to {}",
                archive_sha256_url,
                archive_sha256_file.display()
            );
            Self::download_archive_sha256(&archive_sha256_file, &archive_sha256_url)?;
            // 校验压缩包sha256
            Self::verify_archive_file_sha256(&archive_file, &archive_sha256_file)?;
        }

        // 解压
        log::info!(
            "Unpacking {} to {} ...",
            archive_file.display(),
            version_dest_dir.display()
        );
        if !version_dest_dir.exists() {
            log::debug!("Create version directory: {}", version_dest_dir.display());
            fs::create_dir_all(&version_dest_dir)?
        }
        archive_file
            .to_string_lossy()
            .parse::<Unpack>()?
            .unpack(&version_dest_dir, &archive_file)?;
        // 设置解压成功
        Dir::create_dot_unpacked_success_file(&home, version)?;
        log::info!("{} installed in {}", version, version_dest_dir.display());
        Ok(())
    }

    /// get_upstream_archive_content_length 获取上游压缩包文件长度
    fn get_upstream_archive_content_length(
        version: &str,
        archive_url: &str,
    ) -> Result<u64, anyhow::Error> {
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

    /// download_archive 下载压缩包
    fn download_archive<P: AsRef<Path>>(dest: P, archive_url: &str) -> Result<(), anyhow::Error> {
        let mut response = blocking::get(archive_url)?;
        if !response.status().is_success() {
            return Err(anyhow!("Downloading archive failure"));
        }
        let mut file = File::create(dest)?;
        response.copy_to(&mut file)?;
        Ok(())
    }
    /// download_archive_sha256 下载压缩包sha256
    fn download_archive_sha256<P: AsRef<Path>>(
        dest: P,
        archive_sha256_url: &str,
    ) -> Result<(), anyhow::Error> {
        let mut response = blocking::get(archive_sha256_url)?;
        if !response.status().is_success() {
            return Err(anyhow!("Downloading archive failure"));
        }
        let mut file = File::create(dest)?;
        response.copy_to(&mut file)?;
        Ok(())
    }

    /// compute_file_sha256 计算文件的sha256
    fn compute_file_sha256<P: AsRef<Path>>(path: P) -> Result<String, anyhow::Error> {
        let mut context = Sha256::new();
        let mut file = File::open(path)?;
        let mut buffer = [0; 4096]; // 定义一个缓冲区来处理字节流数据
        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            context.update(&buffer[..bytes_read]);
        }
        Ok(format!("{:x}", context.finalize()))
    }

    /// verify_archive_file_sha256 校验文件压缩包的sha256
    fn verify_archive_file_sha256<P1: AsRef<Path>, P2: AsRef<Path>>(
        archive_file: P1,
        archive_sha256_file: P2,
    ) -> Result<(), anyhow::Error> {
        let expect_sha256 = fs::read_to_string(archive_sha256_file)?;
        let expect_sha256 = expect_sha256.trim();
        let got_sha256 = Self::compute_file_sha256(&archive_file)?;
        if expect_sha256 != got_sha256 {
            return Err(anyhow!(
                "{} corrupt? does not have expected SHA-256 of {}",
                archive_file.as_ref().display(),
                expect_sha256,
            ));
        }
        Ok(())
    }
}
