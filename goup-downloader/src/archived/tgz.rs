use std::fs;
use std::{fs::File, path::Path};

use anyhow::anyhow;
use flate2::read::GzDecoder;
use tar::Archive;

use super::Unpacker;

/// archive *.tar.gz
pub(crate) struct Tgz;

impl Unpacker for Tgz {
    fn unpack<P1, P2>(dest_dir: P1, archive_file: P2) -> Result<(), anyhow::Error>
    where
        P1: AsRef<Path>,
        P2: AsRef<Path>,
    {
        let mut archive = Archive::new(GzDecoder::new(File::open(archive_file)?));
        for entry in archive.entries()? {
            let mut entry = entry?;
            let path = entry.path()?;

            let dest_file = dest_dir.as_ref().join(path);
            let parent = dest_file.parent().ok_or(anyhow!("No parent path found"))?;
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
            entry.unpack(dest_file)?;
        }
        Ok(())
    }
}
