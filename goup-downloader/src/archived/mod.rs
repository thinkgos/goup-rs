mod tgz;
mod zip;

use std::path::Path;

use anyhow::anyhow;

use tgz::Tgz;
use zip::Zip;

pub trait Unpacker {
    fn unpack<P1, P2>(dest_dir: P1, archive_file: P2) -> Result<(), anyhow::Error>
    where
        P1: AsRef<Path>,
        P2: AsRef<Path>;
}

/// Unpack unpacks the provided archive zip or tar.gz file to targetDir.
pub(crate) struct Unpack;

impl Unpacker for Unpack {
    fn unpack<P1, P2>(dest_dir: P1, archive_file: P2) -> Result<(), anyhow::Error>
    where
        P1: AsRef<Path>,
        P2: AsRef<Path>,
    {
        let p = archive_file.as_ref().to_string_lossy();
        if p.ends_with(".zip") {
            Zip::unpack(dest_dir, archive_file)
        } else if p.ends_with(".tar.gz") {
            Tgz::unpack(dest_dir, archive_file)
        } else {
            Err(anyhow!("unsupported archive file"))
        }
    }
}
