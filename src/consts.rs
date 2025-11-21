use std::env;

// env key
pub const GOUP_HOME: &str = "GOUP_HOME";
pub const GOUP_GO_VERSION: &str = "GOUP_GO_VERSION";
pub const GOUP_GO_REGISTRY_INDEX: &str = "GOUP_GO_REGISTRY_INDEX";
pub const GOUP_GO_REGISTRY: &str = "GOUP_GO_REGISTRY";
pub const GOUP_GO_SOURCE_GIT_URL: &str = "GOUP_GO_SOURCE_GIT_URL";
// env value
pub const GO_REGISTRY_INDEX: &str = "https://go.dev";
pub const GO_REGISTRY: &str = "https://dl.google.com/go";
pub const GO_SOURCE_GIT_URL: &str = "https://github.com/golang/go";
pub const GO_SOURCE_UPSTREAM_GIT_URL: &str = "https://go.googlesource.com/go";

pub fn go_version() -> Option<String> {
    env::var(GOUP_GO_VERSION).ok().filter(|s| !s.is_empty())
}

pub fn go_registry_index() -> String {
    get_var_or_else(GOUP_GO_REGISTRY_INDEX, || GO_REGISTRY_INDEX.to_owned())
}

pub fn go_registry() -> String {
    get_var_or_else(GOUP_GO_REGISTRY, || GO_REGISTRY.to_owned())
}

pub fn go_source_git_url() -> String {
    get_var_or_else(GOUP_GO_SOURCE_GIT_URL, || GO_SOURCE_GIT_URL.to_owned())
}

pub fn go_source_upstream_git_url() -> String {
    get_var_or_else(GOUP_GO_SOURCE_GIT_URL, || {
        GO_SOURCE_UPSTREAM_GIT_URL.to_owned()
    })
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
    use super::{GO_REGISTRY, GO_REGISTRY_INDEX, GO_SOURCE_GIT_URL, GO_SOURCE_UPSTREAM_GIT_URL};
    use super::{GOUP_GO_REGISTRY, GOUP_GO_REGISTRY_INDEX, GOUP_GO_SOURCE_GIT_URL};
    use super::{go_registry, go_registry_index, go_source_git_url, go_source_upstream_git_url};

    #[test]
    fn test_env_vars_unset() {
        temp_env::with_vars_unset(
            [
                GOUP_GO_REGISTRY_INDEX,
                GOUP_GO_REGISTRY,
                GOUP_GO_SOURCE_GIT_URL,
            ],
            || {
                assert_eq!(go_registry_index(), GO_REGISTRY_INDEX);
                assert_eq!(go_registry(), GO_REGISTRY);
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
                (GOUP_GO_REGISTRY_INDEX, Some(test_go_host)),
                (GOUP_GO_REGISTRY, Some(test_go_download_base_url)),
                (GOUP_GO_SOURCE_GIT_URL, Some(test_go_source_git_url)),
            ],
            || {
                assert_eq!(go_registry_index(), test_go_host);
                assert_eq!(go_registry(), test_go_download_base_url);
                assert_eq!(go_source_git_url(), test_go_source_git_url);
                assert_eq!(go_source_upstream_git_url(), test_go_source_git_url);
            },
        )
    }
}
