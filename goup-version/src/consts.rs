use std::env;

pub const GOUP_HOME: &str = "GOUP_HOME";
pub const GOUP_GO_HOST: &str = "GOUP_GO_HOST";
pub const GOUP_GO_DOWNLOAD_BASE_URL: &str = "GOUP_GO_DOWNLOAD_BASE_URL";
pub const GOUP_GO_SOURCE_GIT_URL: &str = "GOUP_GO_SOURCE_GIT_URL";
pub const GO_HOST: &str = "https://golang.google.cn"; // "https://go.dev";
pub const GO_DOWNLOAD_BASE_URL: &str = "https://dl.google.com/go";
pub const GO_SOURCE_GIT_URL: &str = "https://github.com/golang/go";
pub const GO_SOURCE_UPSTREAM_GIT_URL: &str = "https://go.googlesource.com/go";

pub fn go_host() -> String {
    get_var_or_else(GOUP_GO_HOST, || GO_HOST.to_owned())
}

pub fn go_download_base_url() -> String {
    get_var_or_else(GOUP_GO_DOWNLOAD_BASE_URL, || {
        GO_DOWNLOAD_BASE_URL.to_owned()
    })
}

pub fn go_source_git_url() -> String {
    get_var_or_else(GOUP_GO_SOURCE_GIT_URL, || GO_SOURCE_GIT_URL.to_owned())
}

pub fn go_source_upstream_git_url() -> String {
    get_var_or_else(GOUP_GO_SOURCE_GIT_URL, || {
        GO_SOURCE_UPSTREAM_GIT_URL.to_owned()
    })
}

/// go_version_archive returns the zip or tar.gz of the given Go version.
/// go1.21.5.linux-amd64.tar.gz, go1.21.5.windows-amd64.zip
pub fn go_version_archive(version: &str) -> String {
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
pub fn archive_sha256(archive_filename: &str) -> String {
    format!("{archive_filename}.sha256")
}

/// archive_url returns returns the zip or tar.gz URL of the given Go version.
#[inline]
pub fn archive_url(archive_filename: &str) -> (String, String) {
    let host = go_download_base_url();
    (
        format!("{host}/{archive_filename}"),
        format!("{host}/{archive_filename}.sha256"),
    )
}

#[inline]
fn get_var_or_else(key: &str, op: impl FnOnce() -> String) -> String {
    env::var(key)
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(op)
}

#[cfg(test)]
mod tests {
    use crate::consts::{archive_sha256, archive_url, go_version_archive};

    use super::{GO_DOWNLOAD_BASE_URL, GO_HOST, GO_SOURCE_GIT_URL, GO_SOURCE_UPSTREAM_GIT_URL};
    use super::{GOUP_GO_DOWNLOAD_BASE_URL, GOUP_GO_HOST, GOUP_GO_SOURCE_GIT_URL};
    use super::{go_download_base_url, go_host, go_source_git_url, go_source_upstream_git_url};

    #[test]
    fn test_env_vars_unset() {
        temp_env::with_vars_unset(
            [
                GOUP_GO_HOST,
                GOUP_GO_DOWNLOAD_BASE_URL,
                GOUP_GO_SOURCE_GIT_URL,
            ],
            || {
                assert_eq!(go_host(), GO_HOST);
                assert_eq!(go_download_base_url(), GO_DOWNLOAD_BASE_URL);
                assert_eq!(go_source_git_url(), GO_SOURCE_GIT_URL);
                assert_eq!(go_source_upstream_git_url(), GO_SOURCE_UPSTREAM_GIT_URL);
            },
        )
    }
    #[test]
    fn test_env_vars_set() {
        let test_go_host = "https://golang.google.cn";
        let test_go_download_base_url = "https://golang.google.cn/dl";
        let test_go_source_git_url = "https://go.googlesource.com/go";
        temp_env::with_vars(
            [
                (GOUP_GO_HOST, Some(test_go_host)),
                (GOUP_GO_DOWNLOAD_BASE_URL, Some(test_go_download_base_url)),
                (GOUP_GO_SOURCE_GIT_URL, Some(test_go_source_git_url)),
            ],
            || {
                assert_eq!(go_host(), test_go_host);
                assert_eq!(go_download_base_url(), test_go_download_base_url);
                assert_eq!(go_source_git_url(), test_go_source_git_url);
                assert_eq!(go_source_upstream_git_url(), test_go_source_git_url);
            },
        )
    }

    #[test]
    fn test_archive() {
        const TEST_VERSION: &str = "1.21.5";
        let archive_filename = go_version_archive(TEST_VERSION);
        #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
        assert_eq!(
            archive_filename,
            format!("{}.darwin-amd64.tar.gz", TEST_VERSION)
        );
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        assert_eq!(
            archive_filename,
            format!("{}.darwin-arm64.tar.gz", TEST_VERSION)
        );

        #[cfg(all(target_os = "linux", target_arch = "x86"))]
        assert_eq!(
            archive_filename,
            format!("{}.linux-386.tar.gz", TEST_VERSION)
        );
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        assert_eq!(
            archive_filename,
            format!("{}.linux-amd64.tar.gz", TEST_VERSION)
        );
        #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
        assert_eq!(
            archive_filename,
            format!("{}.linux-arm64.tar.gz", TEST_VERSION)
        );
        #[cfg(all(target_os = "linux", target_arch = "arm"))]
        assert_eq!(
            archive_filename,
            format!("{}.linux-armv6l.tar.gz", TEST_VERSION)
        );
        #[cfg(all(target_os = "windows", target_arch = "x86"))]
        assert_eq!(
            archive_filename,
            format!("{}.windows-386.zip", TEST_VERSION)
        );
        #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
        assert_eq!(
            archive_filename,
            format!("{}.windows-amd64.zip", TEST_VERSION)
        );

        assert!(archive_sha256(&archive_filename).ends_with(".sha256"));

        let (archive_url, archive_sha256_url) = archive_url(&archive_filename);
        assert!(archive_url.starts_with(&format!("https://dl.google.com/go/{}", TEST_VERSION)));
        assert!(
            archive_sha256_url.starts_with(&format!("https://dl.google.com/go/{}", TEST_VERSION))
        );
    }
}
