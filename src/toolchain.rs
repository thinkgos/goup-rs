use std::str::FromStr;

use anyhow::Result;
use semver::{Op, Version, VersionReq};

/// support toolchain
#[derive(Debug, Eq, Hash, PartialEq)]
pub enum Toolchain {
    Stable,
    Unstable,
    Beta,
    Version(String),
    Nightly,
}

impl FromStr for Toolchain {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "stable" => Self::Stable,
            "unstable" => Self::Unstable,
            "nightly" | "tip" | "gotip" => Self::Nightly,
            "beta" => Self::Beta,
            _ => Self::Version(s.to_owned()),
        })
    }
}

/// a toolchain filter.
#[derive(Debug, Eq, Hash, PartialEq)]
pub enum ToolchainFilter {
    Stable,
    Unstable,
    Beta,
    Filter(String),
}

impl FromStr for ToolchainFilter {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "stable" => Self::Stable,
            "unstable" => Self::Unstable,
            "beta" => Self::Beta,
            _ => Self::Filter(s.to_owned()),
        })
    }
}

/// normalize the version string.
/// 1.21.1   -> go1.21.1
/// go1.21.1 -> go1.21.1
/// tip      -> gotip
/// gotip    -> gotip
pub fn normalize(ver: &str) -> String {
    if ver.starts_with("go") {
        ver.to_string()
    } else {
        format!("go{ver}")
    }
}
/// semantic go version string.
/// 1           -> 1.0.0
/// 1.21        -> 1.21.0
/// 1.21rc2     -> 1.21.0-rc2
/// 1.21.1rc2   -> 1.21.1-rc2
/// 1.21-rc2    -> 1.21.0-rc2
/// 1.21.1-rc2  -> 1.21.1-rc2
/// 1.21.1      -> 1.21.1
pub fn semantic(ver: &str) -> Result<Version> {
    let count_dot = |name: &str| name.chars().filter(|&v| v == '.').count();
    let name = ver
        .find("alpha")
        .or_else(|| ver.find("beta"))
        .or_else(|| ver.find("rc"))
        .map_or_else(
            || match count_dot(ver) {
                0 => format!("{ver}.0.0"),
                1 => format!("{ver}.0"),
                _ => ver.to_string(),
            },
            |idx| {
                let start = &ver[..idx].trim_end_matches('-');
                if count_dot(start) == 2 {
                    format!("{}-{}", start, &ver[idx..])
                } else {
                    format!("{}.0-{}", start, &ver[idx..])
                }
            },
        );
    Ok(Version::parse(&name)?)
}

/// is_match_archived 是否版本请求匹配归档版本.
pub fn is_match_archived(latest: &Version, secondary: &Version, ver_req: &VersionReq) -> bool {
    if ver_req.comparators.len() != 1 {
        return false;
    }
    let comp = ver_req.comparators.first().unwrap();
    if !comp.pre.is_empty() {
        return false;
    }
    let major = comp.major;
    match comp.op {
        // = 精确匹配
        //  =1.24.1 精确匹配
        //  =1.24 匹配1.24.*
        //  =1 匹配1.*.*
        Op::Exact => match (comp.minor, comp.patch) {
            // =1.24.1
            (Some(minor), Some(patch)) => {
                // 主版本号 < 次新版本号
                // 主版本号 == 次新版本号 且 (次版号比次新版本的小 或则 次版本号一致, 修订版版本 < 次修订版本号)
                // 主版本号 == 最新版本号 且 次版号 == 最新版本号 且 修订版版本 <= 最新版本号
                major < secondary.major
                    || (major == secondary.major
                        && (minor < secondary.minor
                            || (minor == secondary.minor && patch <= secondary.patch)))
                    || (major == latest.major && minor == latest.minor && patch <= latest.patch)
            }
            // =1.24
            (Some(minor), None) => {
                major < secondary.major || (major == secondary.major && minor < secondary.minor)
            }
            (None, None) => major < secondary.major,
            _ => false,
        },
        // <= 小于等于
        //  <=1.24.1 匹配 <=1.24.1
        //  <=1.24   匹配 1.24.*
        //  <=1      匹配 1.*.*
        Op::LessEq => match (comp.minor, comp.patch) {
            // <=1.24.1
            (Some(minor), Some(patch)) => {
                // 主版本号 < 次新版本号
                // 主版本号 == 次新版本号 且 (次版号比次新版本的小 或则 次版本号一致, 修订版版本 < 次修订版本号)
                // 主版本号 == 最新版本号 且 次版号 == 最新版本号 且 修订版版本 <= 最新版本号
                major < secondary.major
                    || (major == secondary.major
                        && (minor < secondary.minor
                            || (minor == secondary.minor && patch <= secondary.patch)))
                    || (major == latest.major && minor == latest.minor && patch <= latest.patch)
            }
            // <=1.24
            (Some(minor), None) => {
                major < secondary.major || (major == secondary.major && minor < secondary.minor)
            }
            (None, None) => major < secondary.major,
            _ => false,
        },
        // < 小于
        //  <1.24.1 匹配 <1.24.1
        //  <1.24   匹配 <1.24.0即(1.23.*)
        //  <1      匹配 <1.0.0即(0.*.*)
        Op::Less => match (comp.minor, comp.patch) {
            (Some(minor), Some(patch)) => {
                major < secondary.major
                    || (major == secondary.major
                        && (minor < secondary.minor
                            || (minor == secondary.minor && (patch - 1) <= secondary.patch)))
                    || (major == latest.major
                        && minor == latest.minor
                        && (patch - 1) <= latest.patch)
            }
            (Some(minor), None) => {
                major < secondary.major || (major == secondary.major && minor - 1 < secondary.minor)
            }
            (None, None) => major - 1 < secondary.major,
            _ => false,
        },
        // * 通配符
        //  1.24.* 匹配 1.24.*
        //  1.*    匹配 1.*.*
        //  *      匹配 最新版本
        Op::Wildcard => {
            if let Some(minor) = comp.minor {
                major < secondary.major || (major == secondary.major && minor < secondary.minor)
            } else {
                major < secondary.major
            }
        }
        // ~ 不改变主次版本的最新版本
        //  ~1.24.1 匹配1.24.*
        //  ~1.24   匹配1.24.*
        //  ~1      匹配1.*.*
        Op::Tilde => {
            if let Some(minor) = comp.minor {
                major < secondary.major || (major == secondary.major && minor < secondary.minor)
            } else {
                major < secondary.major
            }
        }
        // ^ 不改变主版本的最新版本
        //  ^1.24.1, ^1.24, ^1 均匹配 1.*.*
        //  1.24.1, 1.24, 1 均匹配 1.*.*
        Op::Caret => comp.major < secondary.major,
        _ => false, // Op::Greater | Op::GreaterEq
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize() {
        assert_eq!(normalize("1.21.1"), "go1.21.1",);
        assert_eq!(normalize("go1.21.1"), "go1.21.1",);
        assert_eq!(normalize("tip"), "gotip",);
        assert_eq!(normalize("gotip"), "gotip",);
    }

    #[test]
    fn test_semantic() {
        assert_eq!(semantic("1").unwrap(), "1.0.0".parse::<Version>().unwrap(),);
        assert_eq!(
            semantic("1.21").unwrap(),
            "1.21.0".parse::<Version>().unwrap(),
        );
        assert_eq!(
            semantic("1.21rc2").unwrap(),
            "1.21.0-rc2".parse::<Version>().unwrap(),
        );
        assert_eq!(
            semantic("1.21.1rc2").unwrap(),
            "1.21.1-rc2".parse::<Version>().unwrap(),
        );
        assert_eq!(
            semantic("1.21-rc2").unwrap(),
            "1.21.0-rc2".parse::<Version>().unwrap(),
        );
        assert_eq!(
            semantic("1.21.1-rc2").unwrap(),
            "1.21.1-rc2".parse::<Version>().unwrap(),
        );
        assert_eq!(
            semantic("1.21.1").unwrap(),
            "1.21.1".parse::<Version>().unwrap(),
        );
    }

    #[test]
    fn test_all_go_version_semantic() {
        let go_versions = [
            "1",
            "1.2.2",
            "1.3rc1",
            "1.3rc2",
            "1.3",
            "1.3.1",
            "1.3.2",
            "1.3.3",
            "1.4beta1",
            "1.4rc1",
            "1.4rc2",
            "1.4",
            "1.4.1",
            "1.4.2",
            "1.4.3",
            "1.5beta1",
            "1.5beta2",
            "1.5beta3",
            "1.5rc1",
            "1.5",
            "1.5.1",
            "1.5.2",
            "1.5.3",
            "1.5.4",
            "1.6beta1",
            "1.6beta2",
            "1.6rc1",
            "1.6rc2",
            "1.6",
            "1.6.1",
            "1.6.2",
            "1.6.3",
            "1.6.4",
            "1.7beta1",
            "1.7beta2",
            "1.7rc1",
            "1.7rc2",
            "1.7rc3",
            "1.7rc4",
            "1.7rc5",
            "1.7rc6",
            "1.7",
            "1.7.1",
            "1.7.3",
            "1.7.4",
            "1.7.5",
            "1.7.6",
            "1.8beta1",
            "1.8beta2",
            "1.8rc1",
            "1.8rc2",
            "1.8rc3",
            "1.8",
            "1.8.1",
            "1.8.2",
            "1.8.3",
            "1.8.4",
            "1.8.5",
            "1.8.6",
            "1.8.7",
            "1.9beta1",
            "1.9beta2",
            "1.9rc1",
            "1.9rc2",
            "1.9",
            "1.9.1",
            "1.9.2rc2",
            "1.9.2",
            "1.9.3",
            "1.9.4",
            "1.9.5",
            "1.9.6",
            "1.9.7",
            "1.10beta1",
            "1.10beta2",
            "1.10rc1",
            "1.10rc2",
            "1.10",
            "1.10.1",
            "1.10.2",
            "1.10.3",
            "1.10.4",
            "1.10.5",
            "1.10.6",
            "1.10.7",
            "1.10.8",
            "1.11beta1",
            "1.11beta2",
            "1.11beta3",
            "1.11rc1",
            "1.11rc2",
            "1.11",
            "1.11.1",
            "1.11.2",
            "1.11.3",
            "1.11.4",
            "1.11.5",
            "1.11.6",
            "1.11.7",
            "1.11.8",
            "1.11.9",
            "1.11.10",
            "1.11.11",
            "1.11.12",
            "1.11.13",
            "1.12beta1",
            "1.12beta2",
            "1.12rc1",
            "1.12",
            "1.12.1",
            "1.12.2",
            "1.12.3",
            "1.12.4",
            "1.12.5",
            "1.12.6",
            "1.12.7",
            "1.12.8",
            "1.12.9",
            "1.12.10",
            "1.12.11",
            "1.12.12",
            "1.12.13",
            "1.12.14",
            "1.12.15",
            "1.12.16",
            "1.12.17",
            "1.13beta1",
            "1.13rc1",
            "1.13rc2",
            "1.13",
            "1.13.1",
            "1.13.2",
            "1.13.3",
            "1.13.4",
            "1.13.5",
            "1.13.6",
            "1.13.7",
            "1.13.8",
            "1.13.9",
            "1.13.10",
            "1.13.11",
            "1.13.12",
            "1.13.13",
            "1.13.14",
            "1.13.15",
            "1.14beta1",
            "1.14rc1",
            "1.14",
            "1.14.1",
            "1.14.2",
            "1.14.3",
            "1.14.4",
            "1.14.5",
            "1.14.6",
            "1.14.7",
            "1.14.8",
            "1.14.9",
            "1.14.10",
            "1.14.11",
            "1.14.12",
            "1.14.13",
            "1.14.14",
            "1.14.15",
            "1.15beta1",
            "1.15rc1",
            "1.15rc2",
            "1.15",
            "1.15.1",
            "1.15.2",
            "1.15.3",
            "1.15.4",
            "1.15.5",
            "1.15.6",
            "1.15.7",
            "1.15.8",
            "1.15.9",
            "1.15.10",
            "1.15.11",
            "1.15.12",
            "1.15.13",
            "1.15.14",
            "1.15.15",
            "1.16beta1",
            "1.16rc1",
            "1.16",
            "1.16.1",
            "1.16.2",
            "1.16.3",
            "1.16.4",
            "1.16.5",
            "1.16.6",
            "1.16.7",
            "1.16.8",
            "1.16.9",
            "1.16.10",
            "1.16.11",
            "1.16.12",
            "1.16.13",
            "1.16.14",
            "1.16.15",
            "1.17beta1",
            "1.17rc1",
            "1.17rc2",
            "1.17",
            "1.17.1",
            "1.17.2",
            "1.17.3",
            "1.17.4",
            "1.17.5",
            "1.17.6",
            "1.17.7",
            "1.17.8",
            "1.17.9",
            "1.17.10",
            "1.17.11",
            "1.17.12",
            "1.17.13",
            "1.18beta1",
            "1.18beta2",
            "1.18rc1",
            "1.18",
            "1.18.1",
            "1.18.2",
            "1.18.3",
            "1.18.4",
            "1.18.5",
            "1.18.6",
            "1.18.7",
            "1.18.8",
            "1.18.9",
            "1.18.10",
            "1.19beta1",
            "1.19rc1",
            "1.19rc2",
            "1.19",
            "1.19.1",
            "1.19.2",
            "1.19.3",
            "1.19.4",
            "1.19.5",
            "1.19.6",
            "1.19.7",
            "1.19.8",
            "1.19.9",
            "1.19.10",
            "1.19.11",
            "1.19.12",
            "1.19.13",
            "1.20rc1",
            "1.20rc2",
            "1.20rc3",
            "1.20",
            "1.20.1",
            "1.20.2",
            "1.20.3",
            "1.20.4",
            "1.20.5",
            "1.20.6",
            "1.20.7",
            "1.20.8",
            "1.20.9",
            "1.20.10",
            "1.20.11",
            "1.20.12",
            "1.20.13",
            "1.20.14",
            "1.21rc2",
            "1.21rc3",
            "1.21rc4",
            "1.21.0",
            "1.21.1",
            "1.21.2",
            "1.21.3",
            "1.21.4",
            "1.21.5",
            "1.21.6",
            "1.21.7",
            "1.21.8",
            "1.21.9",
            "1.21.10",
            "1.21.12",
            "1.21.13",
            "1.22rc1",
            "1.22rc2",
            "1.22.0",
            "1.22.1",
            "1.22.2",
            "1.22.3",
            "1.22.4",
            "1.22.5",
            "1.22.6",
            "1.22.7",
            "1.22.8",
            "1.22.9",
            "1.22.10",
            "1.22.11",
            "1.23rc1",
            "1.23rc2",
            "1.23.0",
            "1.23.1",
            "1.23.2",
            "1.23.3",
            "1.23.4",
            "1.23.5",
            "1.23.6",
            "1.24rc1",
            "1.24rc2",
            "1.24rc3",
            "1.24.0",
            "1.24.1",
            "1.24.2",
            "1.24.3",
            "1.24.4",
            "1.24.5",
            "1.24.6",
            "1.24.7",
            "1.24.8",
            "1.24.9",
            "1.24.10",
            "1.25rc1",
            "1.25rc2",
            "1.25rc3",
            "1.25.0",
            "1.25.1",
            "1.25.2",
            "1.25.3",
            "1.25.4",
        ];
        for ver in go_versions {
            assert!(semantic(ver).is_ok())
        }
    }

    #[test]
    fn test_is_match_archived() -> Result<(), anyhow::Error> {
        let latest = semantic("1.25.2")?;
        let secondary = semantic("1.24.2")?;
        {
            // = 精确匹配
            assert!(is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("=0.9.9")?
            ));
            assert!(is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("=1.23.9")?
            ));
            assert!(is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("=1.24.2")?
            ));
            assert!(!is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("=1.24.3")?
            ));
            assert!(!is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("=1.24.9")?
            ));
            assert!(is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("=1.25.2")?
            ));
            assert!(!is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("=1.25.3")?
            ));
            assert!(!is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("=1.25.9")?
            ));

            assert!(is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("=0.9")?
            ));
            assert!(is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("=1.23")?
            ));
            assert!(!is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("=1.24")?
            ));
            assert!(!is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("=1.25")?
            ));

            assert!(is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("=0")?
            ));
            assert!(!is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("=1")?
            ));
        }

        {
            // ~ 波浪匹配
            assert!(is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("~0.9.9")?
            ));
            assert!(is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("~1.23.9")?
            ));
            assert!(!is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("~1.24.2")?
            ));
            assert!(!is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("~1.24.3")?
            ));
            assert!(!is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("~1.24.9")?
            ));
            assert!(!is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("~1.25.2")?
            ));
            assert!(!is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("~1.25.3")?
            ));
            assert!(!is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("~1.25.9")?
            ));

            assert!(is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("~0.9")?
            ));
            assert!(is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("~1.23")?
            ));
            assert!(!is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("~1.24")?
            ));
            assert!(!is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("~1.25")?
            ));

            assert!(is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("~0")?
            ));
            assert!(!is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("~1")?
            ));
        }

        {
            // <= 小于等于匹配
            assert!(is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("<=0.9.9")?
            ));

            assert!(is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("<=1.23.9")?
            ));
            assert!(is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("<=1.25.2")?
            ));
            assert!(!is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("<=1.25.3")?
            ));
            assert!(!is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("<=1.25.9")?
            ));
            assert!(is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("<=1.24.2")?
            ));
            assert!(!is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("<=1.24.3")?
            ));
            assert!(!is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("<=1.24.9")?
            ));

            assert!(is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("<=0.9")?
            ));
            assert!(is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("<=1.23")?
            ));
            assert!(!is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("<=1.24")?
            ));
            assert!(!is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("<=1.25")?
            ));

            assert!(is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("<=0")?
            ));
            assert!(!is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("<=1")?
            ));
        }

        {
            // < 小于匹配
            assert!(is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("<0.9.9")?
            ));
            assert!(is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("<1.23.2")?
            ));
            assert!(is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("<1.24.3")?
            ));
            assert!(!is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("<1.24.4")?
            ));
            assert!(!is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("<1.24.9")?
            ));
            assert!(is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("<1.25.2")?
            ));
            assert!(is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("<1.25.3")?
            ));
            assert!(!is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("<1.25.4")?
            ));
            assert!(!is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("<1.25.9")?
            ));

            assert!(is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("<0.9")?
            ));
            assert!(is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("<1.23")?
            ));
            assert!(is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("<1.24")?
            ));
            assert!(!is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("<1.25")?
            ));

            assert!(is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("<1")?
            ));
            assert!(!is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("<2")?
            ));
        }

        {
            // * 通配符匹配
            assert!(is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("0.9.*")?
            ));
            assert!(is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("1.23.*")?
            ));
            assert!(!is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("1.24.*")?
            ));
            assert!(!is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("1.25.*")?
            ));

            assert!(is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("0.*")?
            ));
            assert!(!is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("1.*")?
            ));
        }

        {
            // ^ 匹配
            assert!(is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("^0.9.9")?
            ));
            assert!(!is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("^1.23.3")?
            ));
            assert!(!is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("^1.24.3")?
            ));
            assert!(!is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("^1.25.3")?
            ));
            assert!(is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("0.9.9")?
            ));
            assert!(!is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("1.23.3")?
            ));
            assert!(!is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("1.24.3")?
            ));
            assert!(!is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("1.25.3")?
            ));

            assert!(is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("^0.9")?
            ));
            assert!(!is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("^1.23")?
            ));
            assert!(!is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("^1.24")?
            ));
            assert!(!is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("^1.25")?
            ));
            assert!(is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("0.9")?
            ));
            assert!(!is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("1.23")?
            ));
            assert!(!is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("1.24")?
            ));
            assert!(!is_match_archived(
                &latest,
                &secondary,
                &VersionReq::parse("1.25")?
            ));
        }

        Ok(())
    }
}
