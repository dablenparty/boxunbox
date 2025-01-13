use std::{path::PathBuf, str::FromStr};

use anyhow::Context;
use clap::{Parser, ValueHint};
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
pub fn parse_and_expand_pathbuf(s: &str) -> Result<PathBuf, String> {
    let expanded = if let Some(rest) = s.strip_prefix('~') {
        // strip leading `/` if it exists, otherwise it will be interpreted as root when
        // `push`ing to a `PathBuf`
        let rest = rest.strip_prefix('/').unwrap_or(rest);
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
pub struct BoxUnboxArgs {
    /// Package to `box` or `unbox`. Can be a single file or a directory.
    #[arg(value_parser = parse_and_expand_pathbuf, value_hint = ValueHint::AnyPath)]
    pub package: PathBuf,
    /// Target directory where the symlinks are stored. Must be a directory.
    #[arg(short, long, default_value = "~", value_parser = parse_and_expand_pathbuf, value_hint = ValueHint::DirPath)]
    pub target: PathBuf,
    /// Include directories.
    #[arg(short = 'd', long, default_value_t = false)]
    pub include_dirs: bool,
}
