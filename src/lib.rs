use std::{fs::DirEntry, path::PathBuf, str::FromStr};

pub mod cli;
pub mod package;
pub mod rc;

#[derive(Debug)]
pub struct PackageEntry {
    pub fs_entry: DirEntry,
}

impl From<DirEntry> for PackageEntry {
    fn from(value: DirEntry) -> Self {
        Self { fs_entry: value }
    }
}

pub fn expand_and_clean_pathbuf(s: &str) -> anyhow::Result<PathBuf> {
    let expanded = shellexpand::full(s)?;
    let connected = std::env::current_dir()?.join(PathBuf::from_str(&expanded).unwrap());
    Ok(path_clean::clean(connected))
}
