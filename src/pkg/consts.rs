use std::env;

const GO_HOST: &str = "go.dev";
const GO_DOWNLOAD_BASE_URL: &str = "https://dl.google.com/go";
const GO_SOURCE_GIT_URL: &str = "https://github.com/golang/go";
const GO_SOURCE_UPSTREAM_GIT_URL: &str = "https://go.googlesource.com/go";

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

fn get_var_or_else(key: &str, op: impl FnOnce() -> String) -> String {
    env::var(key)
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(op)
}
