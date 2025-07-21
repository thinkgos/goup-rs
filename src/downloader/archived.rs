use std::{path::Path, str::FromStr};

use anyhow::anyhow;

use super::tgz::Tgz;
use super::zip::Zip;

/// unpack format,
/// such as zip,tar.gz.
pub trait Unpacker {
    /// unpack the provided archive file to dest_dir.
    fn unpack<P1, P2>(dest_dir: P1, archive_file: P2) -> Result<(), anyhow::Error>
    where
        P1: AsRef<Path>,
        P2: AsRef<Path>;
}

/// Unpack the provided archive file.
pub(crate) enum Unpack {
    Zip,
    Tgz,
}

impl FromStr for Unpack {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.ends_with(".zip") {
            Ok(Self::Zip)
        } else if s.ends_with(".tar.gz") {
            Ok(Self::Tgz)
        } else {
            Err(anyhow!("unsupported archive file"))
        }
    }
}

impl Unpack {
    pub(crate) fn unpack<P1, P2>(&self, dest_dir: P1, archive_file: P2) -> Result<(), anyhow::Error>
    where
        P1: AsRef<Path>,
        P2: AsRef<Path>,
    {
        match self {
            Unpack::Zip => Zip::unpack(dest_dir, archive_file),
            Unpack::Tgz => Tgz::unpack(dest_dir, archive_file),
        }
    }
}
