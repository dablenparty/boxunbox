use std::path::PathBuf;

use clap::{Parser, ValueHint};
use regex::Regex;

use crate::expand_and_clean_pathbuf;

/// Parses `&str` into a `PathBuf`. If the path begins with a `~`, it is expanded into the users
/// home directory. Also parses environment variables.
///
/// # Arguments
///
/// - `s` - String slice to parse.
///
/// # Errors
///
/// An error will be returned if the path cannot be expanded or the current dir cannot be accessed.
///
/// # Panics
///
/// This function will panic if the users home directory cannot be found OR if `PathBuf::from_str`
/// fails, although neither should ever happen.
pub fn parse_and_expand_pathbuf(s: &str) -> Result<PathBuf, String> {
    expand_and_clean_pathbuf(s).map_err(|err| err.to_string())
}

#[derive(Debug, Parser, Clone)]
pub struct BoxUnboxCli {
    /// Package to `box` or `unbox`. Can be a single file or a directory.
    #[arg(value_parser = parse_and_expand_pathbuf, value_hint = ValueHint::AnyPath)]
    pub package: PathBuf,
    /// Target directory where the symlinks are stored. Must be a directory.
    #[arg(short, long,  value_parser = parse_and_expand_pathbuf, value_hint = ValueHint::DirPath)]
    pub target: Option<PathBuf>,
    /// Include directories.
    #[arg(short = 'd', long)]
    pub include_dirs: Option<bool>,
    /// Dry-run. Do not create/remove any symlinks.
    #[arg(short = None, long, default_value_t = false)]
    pub dry_run: bool,
    /// Ignore file names by passing a regex to this flag. It can be passed multiple times.
    #[arg(short, long = "ignore")]
    pub ignore_pats: Vec<Regex>,
}

// TODO: implement args in the following way:
// if RON file does not exist and no target is given via command line, throw an error
// command line arguments generate a RON file if it does not exist
// command line arguments override RON file and can overwrite it with a flag
