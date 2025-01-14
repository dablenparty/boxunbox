use std::{fs::DirEntry, iter, path::Path};

use anyhow::Context;
use clap::Parser;
use cli::{BoxUnboxArgs, BoxUnboxRcArgs};

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
                // TODO: ignore files based on regex (just like stow)
                if ent.file_name() == ".unboxrc"
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

/// Parse a `.unboxrc` file and return the arguments. Arguments can either be on one line, separate
/// lines, or a combination of the two.
///
/// # Arguments
///
/// - `file_path` - A [`Path`] ref to the RC file.
///
/// # Errors
///
/// An error is returned for any of the following conditions:
///
/// - `file_path` cannot be read (see [`std::fs::read_to_string`]).
/// - `file_path` is not valid Unicode.
/// - The current working directory cannot be determined or changed.
///     - The CWD needs to be changed so that relative paths get canonicalized properly
pub fn parse_rc_file<P: AsRef<Path>>(file_path: P) -> anyhow::Result<BoxUnboxRcArgs> {
    let file_path = file_path.as_ref();

    let rc_text = std::fs::read_to_string(file_path)?;

    // split the args on newlines, then for each line, split on spaces and flatten.
    let combined_args = rc_text
        .split_terminator('\n')
        .map(|s| s.trim())
        .flat_map(|s| s.split_terminator(' '))
        .collect::<Vec<_>>();

    /*
    I use a custom PathBuf parser that expands `~` and canonicalizes the path; however, that
    assumes that the path is being canonicalized from the dierctory the program was called
    from (i.e. the `cwd`). RC files are in the _package_ dirs, so the `cwd` is set to the
    package dir while parsing the RC file, then reset when done.
    */
    let old_cwd = std::env::current_dir()?;
    std::env::set_current_dir(file_path.parent().unwrap())?;

    // prepend the package name since clap requires a prog name to parse args properly.
    // TODO: failure prints usage string, stop that since these aren't actually command line args
    let parsed_args =
        BoxUnboxRcArgs::try_parse_from(iter::once(env!("CARGO_PKG_NAME")).chain(combined_args))
            .with_context(|| format!("Failed to parse args from rc file: {file_path:?}"))?;

    std::env::set_current_dir(old_cwd)?;

    Ok(parsed_args)
}
