use std::{fs::DirEntry, iter, sync::LazyLock};

use anyhow::Context;
use cli::BoxUnboxArgs;
use regex::Regex;

pub mod cli;
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
    static RC_REGEX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new("\\.unboxrc$").expect(".unboxrc regex failed to compile"));

    let BoxUnboxArgs {
        package,
        include_dirs,
        ignore_pats,
        ..
    } = args;

    anyhow::ensure!(package.is_dir(), "{package:?} is not a directory");

    let pkg_entries = package
        .read_dir()
        .with_context(|| format!("Failed to read directory {package:?}"))?
        .filter_map(|res| {
            if let Ok(ref ent) = res {
                // need utf8 string for regex
                let file_name = ent.file_name();
                // shadow previous name to get around temp value error without keeping it
                let file_name = file_name.to_string_lossy();

                if ignore_pats
                    .iter()
                    .chain(iter::once(&RC_REGEX.clone()))
                    .any(|re| re.is_match(&file_name))
                    || (!include_dirs && ent.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
                {
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
