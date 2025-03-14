use std::fs;
use std::{fs::File, io, path::Path};

use anyhow::anyhow;
use zip::ZipArchive;

use super::Unpacker;

/// archive *.zip
pub(crate) struct Zip;

impl Unpacker for Zip {
    fn unpack<P1, P2>(dest_dir: P1, archive_file: P2) -> Result<(), anyhow::Error>
    where
        P1: AsRef<Path>,
        P2: AsRef<Path>,
    {
        let mut archive = ZipArchive::new(File::open(archive_file)?)?;
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let path = file.mangled_name();

            let dest_file = dest_dir.as_ref().join(path);
            if file.is_dir() {
                fs::create_dir_all(&dest_file)?;
                continue;
            }
            let parent = dest_file.parent().ok_or(anyhow!("No parent path found"))?;
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }

            let mut output_file = File::create(dest_file)?;
            io::copy(&mut file, &mut output_file)?;
        }
        Ok(())
    }
}
