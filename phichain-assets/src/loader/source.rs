use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use zip::ZipArchive;

/// A byte-addressable resource pack source, either a filesystem directory or
/// a ZIP archive. Loaders read resources by logical name (e.g. `"tap.png"`);
/// the source hides whether that maps to a path on disk or an entry in a ZIP.
pub enum PackSource {
    Dir(PathBuf),
    Zip(ZipArchive<File>),
}

impl PackSource {
    pub fn open_dir(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        if !path.is_dir() {
            bail!("not a directory: {}", path.display());
        }
        Ok(Self::Dir(path))
    }

    pub fn open_zip(path: &Path) -> Result<Self> {
        let file =
            File::open(path).with_context(|| format!("failed to open {}", path.display()))?;
        let archive = ZipArchive::new(file)
            .with_context(|| format!("failed to read ZIP archive: {}", path.display()))?;
        Ok(Self::Zip(archive))
    }

    pub fn read(&mut self, name: &str) -> Result<Vec<u8>> {
        match self {
            Self::Dir(dir) => {
                let path = dir.join(name);
                std::fs::read(&path).with_context(|| format!("failed to read {}", path.display()))
            }
            Self::Zip(archive) => {
                let mut entry = archive
                    .by_name(name)
                    .with_context(|| format!("missing file in resource pack: {name}"))?;
                let mut buf = Vec::with_capacity(entry.size() as usize);
                entry.read_to_end(&mut buf)?;
                Ok(buf)
            }
        }
    }

    pub fn exists(&self, name: &str) -> bool {
        match self {
            Self::Dir(dir) => dir.join(name).exists(),
            Self::Zip(archive) => archive.file_names().any(|n| n == name),
        }
    }
}
