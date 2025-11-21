use std::{
    collections::BTreeMap,
    env,
    ffi::OsStr,
    fs::{self, File},
    io::{BufRead, BufReader, Read, Write},
    path::Path,
    process::{Command, Stdio},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use anyhow::anyhow;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use regex::Regex;
use reqwest::StatusCode;
use reqwest::blocking::{self, Client};
use reqwest::header::CONTENT_LENGTH;
use semver::{Version, VersionReq};
use serde::Deserialize;
use serde::Serialize;
use sha2::{Digest, Sha256};
use which::which;

use crate::consts;
use crate::dir::Dir;
use crate::{archived::Unpack, toolchain, toolchain::ToolchainFilter};

const HTTP_TIMEOUT: Duration = Duration::from_secs(10);

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct GoFile {
    pub arch: String,
    pub filename: String,
    pub kind: String,
    pub os: String,
    pub sha256: String,
    pub size: isize,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct GoRelease {
    pub version: String,
    pub stable: bool,
    // pub files: Vec<GoFile>,
}

#[derive(Debug)]
pub enum Resolution {
    Resolved(String), // 已确定版本
    Unresolved,       // 未确定, 需要进一步确定
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct LocalGoIndex {
    pub versions: Vec<String>, // 已发布go版本列表
    pub latest: String,        // 最新稳定版本
    pub secondary: String,     // 次新稳定版本
    pub sha256: String,        // 版本列表的sha256
}

impl LocalGoIndex {
    pub fn read() -> Option<LocalGoIndex> {
        let goup_home = Dir::goup_home().ok()?;
        let index_go = goup_home.index_go();
        if index_go.exists() {
            let file = File::open(index_go).ok()?;
            Some(serde_json::from_reader(file).ok()?)
        } else {
            None
        }
    }
    fn write_if_change(index: &LocalGoIndex) -> Result<(), anyhow::Error> {
        let index_go = Dir::goup_home()?.index_go();
        if index_go.exists()
            && let Ok(file) = File::open(&index_go)
            && let Ok(old) = serde_json::from_reader::<_, LocalGoIndex>(file)
            && old.sha256 == index.sha256
        {
            return Ok(());
        }
        let file = File::create(&index_go)?;
        serde_json::to_writer(file, index)?;
        Ok(())
    }
    // 匹配本地版本
    pub fn match_version(&self, ver_req: &VersionReq) -> Option<String> {
        self.versions
            .iter()
            .rev()
            .find_map(|v| {
                toolchain::semantic(v)
                    .ok()
                    .filter(|semver| ver_req.matches(semver))
                    .map(|_| v)
            })
            .map(ToOwned::to_owned)
    }
    // 尝试匹配归档版本
    fn try_match_archived_version(
        &self,
        ver_req: &VersionReq,
    ) -> Result<Resolution, anyhow::Error> {
        if self.versions.is_empty() || self.latest.is_empty() || self.secondary.is_empty() {
            return Ok(Resolution::Unresolved);
        }
        if ver_req.comparators.len() != 1 {
            // 先匹配本地版本
            let ver = self.match_version(ver_req);
            let search_type = if let Some(ver) = ver
                && (ver != self.latest || ver != self.secondary)
            {
                Resolution::Resolved(ver)
            } else {
                Resolution::Unresolved
            };
            return Ok(search_type);
        }

        let latest = toolchain::semantic(&self.latest)?;
        let secondary = toolchain::semantic(&self.secondary)?;
        let is_match_archived = toolchain::is_match_archived(&latest, &secondary, ver_req);
        if is_match_archived {
            self.match_version(ver_req)
                .map(Resolution::Resolved)
                .ok_or_else(|| anyhow!("no matching version found!"))
        } else {
            Ok(Resolution::Unresolved)
        }
    }
}

impl From<Vec<String>> for LocalGoIndex {
    fn from(versions: Vec<String>) -> Self {
        let mut context = Sha256::new();
        // major.minor -> 最新稳定版本
        let mut latest_stable: BTreeMap<(u64, u64), (Version, &str)> = BTreeMap::new();
        for v in versions.iter() {
            context.update(v);
            // 注意跳过 rc/beta/alpha）
            if let Ok(ver) = toolchain::semantic(v)
                && ver.pre.is_empty()
            {
                latest_stable
                    .entry((ver.major, ver.minor))
                    .and_modify(|existing| {
                        if ver > existing.0 {
                            *existing = (ver.clone(), v);
                        }
                    })
                    .or_insert((ver, v));
            }
        }
        let (latest, secondary) = {
            let mut iter = latest_stable.values().rev().take(2).map(|v| v.1);
            let latest = iter.next().unwrap_or_default();
            let second_latest = iter.next().unwrap_or(latest);
            (latest.to_owned(), second_latest.to_owned())
        };
        let sha256 = format!("{:x}", context.finalize());
        Self {
            versions,
            latest,
            secondary,
            sha256,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RegistryIndex {
    host: String,
}

impl RegistryIndex {
    pub fn new(host: &str) -> Self {
        Self {
            host: host.to_owned(),
        }
    }
    /// get upstream latest go version.
    pub fn get_upstream_latest_go_version(&self) -> Result<String, anyhow::Error> {
        let body = Client::builder()
            .timeout(HTTP_TIMEOUT)
            .build()?
            .get(format!("{}/VERSION?m=text", self.host))
            .send()?
            .text()?;
        body.split('\n')
            .nth(0)
            .ok_or_else(|| anyhow!("Getting latest Go version failed"))
            .map(|v| v.to_owned())
    }

    pub fn match_version_req(&self, version_req: &str) -> Result<String, anyhow::Error> {
        log::debug!("version request: {version_req}");
        let ver_req = VersionReq::parse(version_req)?;

        let search_type = LocalGoIndex::read().map_or(Ok(Resolution::Unresolved), |v| {
            v.try_match_archived_version(&ver_req)
        })?;
        if let Resolution::Resolved(ver) = search_type {
            log::debug!("use archived!!!");
            Ok(ver)
        } else {
            log::debug!("use active!!!");
            self.list_upstream_go_versions_filter(None)?
                .iter()
                .rev()
                .find_map(|v| {
                    toolchain::semantic(v)
                        .ok()
                        .filter(|semver| ver_req.matches(semver))
                        .map(|_| v)
                })
                .map(|v| v.to_owned())
                .ok_or_else(|| anyhow!("no matching version found!"))
        }
    }
    /// list upstream go versions filter by toolchain filter.
    /// NOTE: 此方法每次都更新缓存!
    pub fn list_upstream_go_versions_filter(
        &self,
        filter: Option<ToolchainFilter>,
    ) -> Result<Vec<String>, anyhow::Error> {
        let ver = self.list_upstream_go_versions()?;
        LocalGoIndex::write_if_change(&ver.clone().into()).ok();
        let Some(filter) = filter else {
            return Ok(ver);
        };
        let re = match filter {
            ToolchainFilter::Stable => {
                r#"(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)(?:\.(?:0|[1-9]\d*))?\b"#.to_string()
            }
            ToolchainFilter::Unstable => {
                r#"(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)(?:\.(?:0|[1-9]\d*))?(?:rc(?:0|[1-9]\d*))"#
                    .to_string()
            }
            ToolchainFilter::Beta => {
                r#"(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)(?:\.(?:0|[1-9]\d*))?(?:beta(?:0|[1-9]\d*))"#
                    .to_string()
            }
            ToolchainFilter::Filter(s) => format!("(.*{s}.*)"),
        };
        let re = Regex::new(&re)?;
        Ok(ver
            .into_iter()
            .filter_map(|v| re.is_match(&v).then_some(v))
            .collect())
    }

    /// list upstream go versions if get go version failure from http then fallback use git.
    fn list_upstream_go_versions(&self) -> Result<Vec<String>, anyhow::Error> {
        let (tx, rx) = mpsc::channel();
        {
            let tx = tx.clone();
            let this = self.clone();
            thread::spawn(move || {
                let r = this.list_upstream_go_versions_via_http();
                let _ = tx.send(r);
            });
        }

        let thread_count = if which("git").is_ok() {
            let tx = tx.clone();
            thread::spawn(move || {
                let r: Result<Vec<String>, anyhow::Error> =
                    Self::list_upstream_go_versions_via_git();
                let _ = tx.send(r);
            });
            2
        } else {
            1
        };

        let mut last_err = Ok(vec![]);
        for _ in 0..thread_count {
            match rx.recv()? {
                ok @ Ok(_) => return ok,
                Err(e) => last_err = Err(e),
            }
        }
        last_err
    }
    /// list upstream go versions via http.
    fn list_upstream_go_versions_via_http(&self) -> Result<Vec<String>, anyhow::Error> {
        log::trace!("list upstream go versions via http: {:?}", self.host);
        Ok(Client::builder()
            .timeout(HTTP_TIMEOUT)
            .build()?
            .get(format!("{}/dl/?mode=json&include=all", self.host))
            .send()?
            .json::<Vec<GoRelease>>()?
            .into_iter()
            .map(|v| v.version.trim_start_matches("go").to_string())
            .rev()
            .collect())
    }
    /// list upstream go versions via git.
    fn list_upstream_go_versions_via_git() -> Result<Vec<String>, anyhow::Error> {
        let go_source_git_url = consts::go_source_git_url();
        log::trace!("list upstream go versions via git: {:?}", go_source_git_url);
        let output = Command::new("git")
            .args([
                "ls-remote",
                "--sort=version:refname",
                "--tags",
                &go_source_git_url,
            ])
            .output()?
            .stdout;
        Ok(Regex::new("refs/tags/go(.+)")?
            .captures_iter(&String::from_utf8_lossy(&output))
            .map(|capture| capture[1].to_string())
            .collect())
    }
}

pub struct Registry<'a> {
    host: &'a str,
    enable_check_archive_size: bool,
    skip_verify: bool,
}

impl<'a> Registry<'a> {
    pub fn new(host: &'a str, skip_verify: bool, enable_check_archive_size: bool) -> Self {
        Self {
            host,
            enable_check_archive_size,
            skip_verify,
        }
    }

    pub fn install_go(&self, version: &str) -> Result<(), anyhow::Error> {
        let goup_home = Dir::goup_home()?;
        let version_dest_dir = goup_home.version(version);

        let mp = MultiProgress::new();
        let pb = mp.add(ProgressBar::new_spinner());
        pb.enable_steady_tick(Duration::from_millis(100));
        pb.set_message(format!("Installing {version}"));

        // 是否已解压成功并且存在
        if goup_home.is_dot_unpacked_success_file_exists(version) {
            pb.finish_with_message(format!(
                "Already installed {} in {:?}",
                version,
                version_dest_dir.display()
            ));
            return Ok(());
        }
        // download directory
        let dl_dest_dir = goup_home.cache();
        // 压缩包文件名称
        let archive_filename = archive_go_version(version);
        // 压缩包sha256文件名称
        let archive_sha256_filename = archive_sha256(&archive_filename);
        // 压缩包url
        let (archive_url, archive_sha256_url) = archive_url(self.host, &archive_filename);
        if !dl_dest_dir.exists() {
            log::debug!("Create download directory");
            fs::create_dir_all(&dl_dest_dir)?
        }

        // 压缩包文件
        let archive_file = dl_dest_dir.join_path(archive_filename);
        let archive_sha256_file = dl_dest_dir.join_path(archive_sha256_filename);
        if !archive_file.exists() {
            pb.set_message(format!(
                "Downloading archive file from {} to {}",
                archive_url,
                archive_file.display()
            ));
            // 下载压缩包文件
            Self::download_file(&archive_file, &archive_url, Some(&mp))?;

            //  有一些镜像仓库不支持获取压缩包长度, 默认不验证
            if self.enable_check_archive_size {
                log::debug!("Check archive file content length");
                // 压缩包长度
                let archive_content_length =
                    Self::get_archive_content_length(version, &archive_url)?;
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
        }
        if self.skip_verify {
            pb.set_message("Skip verify archive file sha256");
        } else {
            if !archive_sha256_file.exists()
                || Self::verify_archive_file_sha256(&archive_file, &archive_sha256_file).is_err()
            {
                // 下载压缩包sha256
                pb.set_message(format!(
                    "Download archive sha256 file from {} to {}",
                    archive_sha256_url,
                    archive_sha256_file.display()
                ));
                // 下载压缩包sha256文件
                let r = Self::download_file(&archive_sha256_file, &archive_sha256_url, None);
                if r.is_err() {
                    log::warn!(
                        "Download archive sha256 file failure, maybe the version '{version}' miss it, try add option '--skip-verify'",
                    );
                    return r;
                }
            }
            // 校验压缩包sha256
            pb.set_message(format!("Verifying '{}' sha256", archive_file.display()));
            Self::verify_archive_file_sha256(&archive_file, &archive_sha256_file)?;
        }

        // 解压
        pb.set_message(format!(
            "Unpacking {} to {}",
            archive_file.display(),
            version_dest_dir.display()
        ));
        if !version_dest_dir.exists() {
            log::debug!("Create version directory: {}", version_dest_dir.display());
            fs::create_dir_all(&version_dest_dir)?
        }
        archive_file
            .to_string_lossy()
            .parse::<Unpack>()?
            .unpack(&version_dest_dir, &archive_file)?;
        // 设置解压成功标记
        goup_home.create_dot_unpacked_success_file(version)?;
        pb.finish_with_message(format!(
            "Installed {} in {}",
            version,
            version_dest_dir.display()
        ));

        Ok(())
    }

    // get_archive_content_length 获取压缩包文件长度
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

    /// download_file 下载文件
    fn download_file<P: AsRef<Path>>(
        dest: P,
        url: &str,
        mp: Option<&MultiProgress>,
    ) -> Result<(), anyhow::Error> {
        if let Some(mp) = mp {
            let client = Client::new();
            let content_length = client
                .head(url)
                .header("User-Agent", "goup-rs Client")
                .timeout(Duration::from_secs(10))
                .send()?
                .headers()
                .get(CONTENT_LENGTH)
                .ok_or_else(|| anyhow!("no content length header"))?
                .to_str()?
                .parse::<u64>()?;
            let mut dest_file = fs::File::create(dest)?;

            let pb = mp.add(ProgressBar::new(content_length));
            pb.set_style(
                ProgressStyle::default_bar()
                    .template(
                        "  [{elapsed_precise}] [{bar:30.cyan/blue}] {bytes}/{total_bytes} ({eta})",
                    )?
                    .progress_chars("=> "),
            );
            pb.enable_steady_tick(Duration::from_millis(100));

            const CHUNK_SIZE: u64 = 1024 * 1024; // 1MB
            const MAX_CHUNK_SIZE: u64 = 1024 * 1024 * 16; // 16MB

            let mut speed = 0.0;
            let mut chunk_size = 2 * CHUNK_SIZE;
            let mut start = 0;
            while start < content_length {
                let end = start + chunk_size - 1;
                let instant = Instant::now();
                let buf = client
                    .get(url)
                    .header("User-Agent", "GOUP Client")
                    .header("Range", format!("bytes={start}-{end}"))
                    .timeout(Duration::from_secs(30))
                    .send()?
                    .bytes()?;
                let elapsed = instant.elapsed();
                dest_file.write_all(&buf)?;

                let real_chunk_size = buf.len() as u64;
                let real_speed = (real_chunk_size as f32) / elapsed.as_secs_f32();

                start = end + 1;
                speed = if speed == 0.0 {
                    real_speed
                } else {
                    (speed + real_speed) * 0.5
                };
                chunk_size = if speed < real_speed {
                    chunk_size as f32 * 1.75
                } else {
                    chunk_size as f32 * 0.75
                } as u64;
                chunk_size = chunk_size.clamp(CHUNK_SIZE, MAX_CHUNK_SIZE);

                pb.inc(real_chunk_size);
            }

            pb.finish_and_clear();
            mp.remove(&pb);
        } else {
            let mut response = blocking::get(url)?;
            if !response.status().is_success() {
                return Err(anyhow!("Downloading file failure"));
            }
            let mut file = File::create(dest)?;
            response.copy_to(&mut file)?;
        }
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

    /// verify_archive_file_sha256 校验文件sha256是否合法
    fn verify_archive_file_sha256<P1, P2>(
        archive_file: P1,
        archive_sha256_file: P2,
    ) -> Result<(), anyhow::Error>
    where
        P1: AsRef<Path>,
        P2: AsRef<Path>,
    {
        let expect_sha256 = fs::read_to_string(archive_sha256_file)?;
        let expect_sha256 = expect_sha256.trim();
        let computed_sha256 = Self::compute_file_sha256(&archive_file)?;
        if computed_sha256 != expect_sha256 {
            return Err(anyhow!(
                "{} corrupt? does not have expected SHA-256 of {}",
                archive_file.as_ref().display(),
                expect_sha256,
            ));
        }
        Ok(())
    }
}

pub struct NightlyRegistry<'a> {
    _cl: Option<&'a str>,
}

impl<'a> NightlyRegistry<'a> {
    pub fn new(_cl: Option<&'a str>) -> Self {
        Self { _cl }
    }
    pub fn install_go(&self) -> Result<(), anyhow::Error> {
        if which("git").is_err() {
            return Err(anyhow!(
                r#""git" binary not found, make sure it is installed!"#,
            ));
        }

        let gotip_go = Dir::goup_home()?.version("gotip");
        let gotip_git = gotip_go.join_path(".git");
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

        let script = match env::consts::OS {
            "windows" => "make.bat",
            "plan9" => "make.rs",
            _ => "make.bash",
        };
        //* 执行 ./src/<make.bashmake.rs|make.bat> 有$GOROOT/src问题
        //* $HOME/{owner}/.goup/gotip/src/<make.bash|make.rs|make.bat>
        Self::execute_command(
            gotip_go.join("src").join(script),
            gotip_go.join_path("src"),
            [],
        )?;
        Ok(())
    }
    fn execute_command<P, I, S>(program: S, working_dir: P, args: I) -> Result<(), anyhow::Error>
    where
        P: AsRef<Path>,
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let mut command = Command::new(&program)
            .current_dir(working_dir)
            .args(args)
            .stdout(Stdio::piped())
            .spawn()?;
        if let Some(stdout) = command.stdout.take() {
            let reader = BufReader::new(stdout);

            for line in reader.lines().map_while(Result::ok) {
                println!("{line}");
            }
        }
        let status = command.wait()?;
        if !status.success() {
            return Err(anyhow!("Command failed with {}", status));
        }
        Ok(())
    }
}

/// archive_go_version returns the zip or tar.gz of the given Go version.
/// go1.21.5.linux-amd64.tar.gz, go1.21.5.windows-amd64.zip
fn archive_go_version(version: &str) -> String {
    let os = match env::consts::OS {
        "macos" => "darwin",
        os => os,
    };
    let arch = match (os, env::consts::ARCH) {
        (_, "x86") => "386",
        (_, "x86_64") => "amd64",
        ("linux", "arm") => "armv6l",
        (_, "aarch64") => "arm64",
        _ => env::consts::ARCH,
    };
    let ext = if os == "windows" { "zip" } else { "tar.gz" };
    format!("{version}.{os}-{arch}.{ext}")
}

/// archive_sha256 returns `{archive}.sha256`
/// go1.21.5.linux-amd64.tar.gz.sha256, go1.21.5.windows-amd64.zip.sha256
#[inline]
fn archive_sha256(archive_filename: &str) -> String {
    format!("{archive_filename}.sha256")
}

/// archive_url returns returns the zip or tar.gz URL of the given Go version.
#[inline]
fn archive_url(registry: &str, archive_filename: &str) -> (String, String) {
    (
        format!("{registry}/{archive_filename}"),
        format!("{registry}/{archive_filename}.sha256"),
    )
}

#[cfg(test)]
mod tests {
    use super::LocalGoIndex;
    use super::{archive_go_version, archive_sha256, archive_url};

    #[test]
    fn test_cache_go_version_impl_from_vec_trait() {
        {
            let v1 = vec![
                "1.24.0", "1.25.2", "1.24.1", "1.25rc1", "1.25.1", "1.23.2", "1.25.3", "1.24rc1",
                "1.23rc1", "1.23.0", "1.24.2", "1.23.1", "1.25.0",
            ];

            let v2 = v1.iter().map(|s| s.to_string()).collect::<Vec<String>>();
            let cgv: LocalGoIndex = v2.into();
            assert_eq!(cgv.versions, v1);
            assert_eq!(cgv.latest, "1.25.3");
            assert_eq!(cgv.secondary, "1.24.2");
        }
        {
            let v1 = vec![
                "1.24.0",
                "1.24rc1",
                "1.24.2",
                "1.25rc2",
                "1.25beta2",
                "1.23.1",
                "1.25rc1",
                "1.24.1",
                "1.23rc1",
                "1.23.0",
                "1.25beta1",
            ];
            let v2 = v1.iter().map(|s| s.to_string()).collect::<Vec<String>>();
            let cgv: LocalGoIndex = v2.into();
            assert_eq!(cgv.versions, v1);
            assert_eq!(cgv.latest, "1.24.2");
            assert_eq!(cgv.secondary, "1.23.1");
        }
    }

    #[test]
    fn test_archive() {
        const TEST_VERSION: &str = "1.21.5";
        let archive_filename = archive_go_version(TEST_VERSION);
        #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
        assert_eq!(
            archive_filename,
            format!("{TEST_VERSION}.darwin-amd64.tar.gz")
        );
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        assert_eq!(
            archive_filename,
            format!("{TEST_VERSION}.darwin-arm64.tar.gz")
        );

        #[cfg(all(target_os = "linux", target_arch = "x86"))]
        assert_eq!(archive_filename, format!("{TEST_VERSION}.linux-386.tar.gz"));
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        assert_eq!(
            archive_filename,
            format!("{TEST_VERSION}.linux-amd64.tar.gz")
        );
        #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
        assert_eq!(
            archive_filename,
            format!("{TEST_VERSION}.linux-arm64.tar.gz")
        );
        #[cfg(all(target_os = "linux", target_arch = "arm"))]
        assert_eq!(
            archive_filename,
            format!("{TEST_VERSION}.linux-armv6l.tar.gz")
        );
        #[cfg(all(target_os = "windows", target_arch = "x86"))]
        assert_eq!(archive_filename, format!("{TEST_VERSION}.windows-386.zip"));
        #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
        assert_eq!(
            archive_filename,
            format!("{TEST_VERSION}.windows-amd64.zip")
        );

        assert!(archive_sha256(&archive_filename).ends_with(".sha256"));

        let registry = "https://dl.google.com/go";
        let (archive_url, archive_sha256_url) = archive_url(registry, &archive_filename);
        assert!(archive_url.starts_with(&format!("{registry}/{TEST_VERSION}")));
        assert!(archive_sha256_url.starts_with(&format!("{registry}/{TEST_VERSION}")));
    }
}
