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

            if let Some(path_str) = path.to_str() {
                // 标准化路径分隔符并检查是否在 go/ 目录下
                let normalized_path = path_str.replace('\\', "/");
                if let Some(relative_path) = normalized_path.strip_prefix("go/") {
                    let dest_file = dest_dir.as_ref().join(relative_path);
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
            }
        }
        Ok(())
    }
}
