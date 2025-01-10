use std::{
    fs::DirEntry,
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::Context;
use clap::Parser;
use directories_next::UserDirs;

/// Parses `&str` into a `PathBuf`. If the path begins with a `~`, it is expanded into the users
/// home directory.
///
/// # Arguments
///
/// - `s` - String slice to parse.
///
/// # Errors
///
/// An error will be returned if the path cannot be canonicalized.
///
/// # Panics
///
/// This function will panic if the users home directory cannot be found OR if `PathBuf::from_str`
/// fails, although neither should ever happen.
fn parse_and_expand_pathbuf(s: &str) -> Result<PathBuf, String> {
    let expanded = if let Some(rest) = s.strip_prefix('~') {
        let mut p = UserDirs::new()
            .unwrap_or_else(|| unreachable!("Failed to locate user home directory"))
            .home_dir()
            .to_path_buf();
        p.push(rest);
        p
    } else {
        PathBuf::from_str(s).unwrap_or_else(|_| unreachable!("Failed to convert &str to PathBuf"))
    };

    expanded
        .canonicalize()
        .with_context(|| format!("failed to canonicalize: {expanded:?}"))
        .map_err(|e| e.to_string())
}

#[derive(Debug, Parser, Clone)]
struct StowArgs {
    /// Package to stow. Can be a single file or a directory.
    #[arg(value_parser = parse_and_expand_pathbuf)]
    pub package: PathBuf,
    #[arg(default_value = "~", value_parser = parse_and_expand_pathbuf)]
    pub target: PathBuf,
    #[arg(short = 'd', long, default_value_t = false)]
    pub include_dirs: bool,
}

/// Stows a package entry in `target`. The `pkg_entry`'s file name is used for the name of the symlink.
///
/// # Arguments
///
/// - `pkg_entry` - The `DirEntry` to stow.
/// - `target` - The `&Path` to stow in. Must be a directory.
fn stow_package_entry(pkg_entry: &DirEntry, target: &Path) -> anyhow::Result<()> {
    anyhow::ensure!(
        pkg_entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false),
        "`{}`={pkg_entry:?} is not a directory!",
        stringify!(pkg_entry),
    );

    let path = pkg_entry.path();
    let link_path = target.join(pkg_entry.file_name());

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(path, link_path)?;
    }

    #[cfg(windows)]
    {
        todo!("implement windows?")
    }

    #[cfg(all(not(windows), not(unix)))]
    {
        std::fs::soft_link(path, link_path)?;
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let cli_args = StowArgs::parse();

    #[cfg(debug_assertions)]
    println!("{cli_args:#?}");

    let StowArgs {
        include_dirs,
        ref package,
        target,
    } = cli_args;

    if !package.is_dir() {
        anyhow::bail!("{package:?} is not a directory");
    }

    for res in package.read_dir()? {
        match res {
            Ok(entry) => {
                if !include_dirs && entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                    return Ok(());
                }
                if let Err(error) = stow_package_entry(&entry, &target) {
                    eprintln!(
                        "error stowing {}: {error:?}",
                        entry.file_name().to_string_lossy()
                    );
                }
            }
            Err(error) => eprintln!("error reading entry: {error:?}"),
        }
    }

    Ok(())
}
