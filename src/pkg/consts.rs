use std::env;

pub const GO_HOST: &str = "https://go.dev";
pub const GO_DOWNLOAD_BASE_URL: &str = "https://dl.google.com/go";
pub const GO_SOURCE_GIT_URL: &str = "https://github.com/golang/go";
pub const GO_SOURCE_UPSTREAM_GIT_URL: &str = "https://go.googlesource.com/go";

pub fn go_host() -> String {
    get_var_or_else("GOUP_GO_HOST", || GO_HOST.to_owned())
}

pub fn go_download_base_url() -> String {
    get_var_or_else("GOUP_GO_DOWNLOAD_BASE_URL", || {
        GO_DOWNLOAD_BASE_URL.to_owned()
    })
}

pub fn go_source_git_url() -> String {
    get_var_or_else("GOUP_GO_SOURCE_GIT_URL", || GO_SOURCE_GIT_URL.to_owned())
}

pub fn go_source_upstream_git_url() -> String {
    get_var_or_else("GOUP_GO_SOURCE_GIT_URL", || {
        GO_SOURCE_UPSTREAM_GIT_URL.to_owned()
    })
}

// go_version_archive_url returns the zip or tar.gz URL of the given Go version.
pub fn go_version_archive_url(version: &str) -> String {
    let os = env::consts::OS;
    let arch = match (os, env::consts::ARCH) {
        (_, "x86") => "386",
        (_, "x86_64") => "amd64",
        ("linux", "arm") => "armv6l",
        (_, "aarch64") => "arm64",
        _ => env::consts::ARCH,
    };
    let ext = if os == "windows" { "zip" } else { "tar.gz" };
    return format!(
        "{}/{}.{}-{}.{}",
        go_download_base_url(),
        version,
        os,
        arch,
        ext
    );
}

fn get_var_or_else(key: &str, op: impl FnOnce() -> String) -> String {
    env::var(key)
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(op)
}
