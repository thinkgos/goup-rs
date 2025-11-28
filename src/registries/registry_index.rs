pub mod ngx_auto_index;
pub mod ngx_fancy_index;
mod official;
mod official_git;

use std::fmt::Display;
use std::str::FromStr;

use anyhow::anyhow;
use regex::Regex;
use semver::VersionReq;

use self::official::Official;

use crate::consts;
use crate::registries::go_index::{GoIndex, Resolution};
use crate::registries::registry_index::ngx_auto_index::NgxAutoIndex;
use crate::registries::registry_index::ngx_fancy_index::NgxFancyIndex;
use crate::registries::registry_index::official_git::OfficialGit;
use crate::{toolchain, toolchain::ToolchainFilter};

pub trait RegistryIndex {
    /// get upstream latest go version.
    fn get_upstream_latest_go_version(&self) -> Result<String, anyhow::Error>;
    /// list upstream go versions.
    fn list_upstream_go_versions(&self) -> Result<Vec<String>, anyhow::Error>;

    /// match version request.
    /// 1. 尝试先从本地缓存查找, 如果找到, 且是确定的归档版本, 则返回, 否则从上游查找.
    fn match_version_req(&self, version_req: &str) -> Result<String, anyhow::Error> {
        log::debug!("version request: {version_req}");
        let ver_req = VersionReq::parse(version_req)?;

        let search_type = GoIndex::read().map_or(Ok(Resolution::Unresolved), |v| {
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
    /// NOTE: 此方法每次都从上游查找, 并尝试更新本地缓存!
    fn list_upstream_go_versions_filter(
        &self,
        filter: Option<&ToolchainFilter>,
    ) -> Result<Vec<String>, anyhow::Error> {
        let ver = self.list_upstream_go_versions()?;
        GoIndex::write_if_change(&ver.clone().into()).ok();
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
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum RegistryIndexType {
    Official(String),
    OfficialGit(String),
    NgxAutoIndex(String),
    NgxFancyIndex(String),
}
impl RegistryIndexType {
    pub fn as_registry_index(&self) -> Box<dyn RegistryIndex> {
        match self {
            Self::Official(host) => Box::new(Official::new(host)),
            Self::OfficialGit(url) => Box::new(OfficialGit::new(url)),
            Self::NgxAutoIndex(host) => Box::new(NgxAutoIndex::new(host)),
            Self::NgxFancyIndex(host) => Box::new(NgxFancyIndex::new(host)),
        }
    }
}

impl FromStr for RegistryIndexType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (t, host) = match s {
            "git" => ("git", consts::GO_SOURCE_GIT_URL),
            "official" => ("official", consts::GO_REGISTRY_INDEX),
            _ => s.split_once("|").unwrap_or(("official", s)),
        };
        let host = host.to_owned();
        match t {
            "official" => Ok(Self::Official(host)),
            "ngx-auto-index" => Ok(Self::NgxAutoIndex(host)),
            "ngx-fancy-index" => Ok(Self::NgxFancyIndex(host)),
            "git" => Ok(Self::OfficialGit(host)),
            _ => Ok(Self::Official(consts::GO_REGISTRY_INDEX.to_owned())),
        }
    }
}

impl Display for RegistryIndexType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Official(host) => write!(f, "official|{host}"),
            Self::NgxAutoIndex(host) => write!(f, "ngx-auto-index|{host}"),
            Self::NgxFancyIndex(host) => write!(f, "ngx-fancy-index|{host}"),
            Self::OfficialGit(url) => write!(f, "git|{url}"),
        }
    }
}
