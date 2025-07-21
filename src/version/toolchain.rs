use std::str::FromStr;

/// support toolchain
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
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
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
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
