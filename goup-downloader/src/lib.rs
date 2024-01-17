mod archived;
mod downloader;

use std::str::FromStr;

pub use downloader::Downloader;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Toolchain {
    Stable,
    Nightly,
    Beta,
    Semver(String),
}

impl FromStr for Toolchain {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "stable" => Self::Stable,
            "nightly" | "tip" | "gotip" => Self::Nightly,
            "beta" => Self::Beta,
            _ => {
                // TODO: version validation
                Self::Semver(s.to_owned())
            }
        })
    }
}
