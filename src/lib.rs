use std::fs::DirEntry;

use anyhow::Context;
use cli::BoxUnboxArgs;

pub mod cli;

#[derive(Debug)]
pub struct PackageEntry {
    pub fs_entry: DirEntry,
}

impl From<DirEntry> for PackageEntry {
    fn from(value: DirEntry) -> Self {
        Self { fs_entry: value }
    }
}

/// Gets all entries to either `box` or `unbox` from a package.
///
/// # Arguments
///
/// - `args` - Arguments for filtering the packages.
///
/// # Errors
///
/// An error is returned if one occurs when reading the package directory. Errors for individual
/// [`DirEntry`]'s do not end this function and are instead collected into the returned `Vec`.
pub fn get_package_entries(
    args: &BoxUnboxArgs,
) -> anyhow::Result<Vec<anyhow::Result<PackageEntry>>> {
    let BoxUnboxArgs {
        package,
        include_dirs,
        ..
    } = args;

    anyhow::ensure!(package.is_dir(), "{package:?} is not a directory");

    let pkg_entries = package
        .read_dir()
        .with_context(|| format!("Failed to read directory {package:?}"))?
        .filter_map(|res| {
            if let Ok(ref ent) = res {
                if !include_dirs && ent.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                    return None;
                }
            }
            Some(
                res.map(PackageEntry::from)
                    .context("Failed to read dir entry"),
            )
        })
        .collect();

    Ok(pkg_entries)
}
