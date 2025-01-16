use std::{path::PathBuf, str::FromStr};

use clap::{Parser, ValueHint};
use regex::Regex;

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
    fn expand_and_clean(s: &str) -> anyhow::Result<PathBuf> {
        let expanded = shellexpand::full(s)?;
        let connected = std::env::current_dir()?.join(PathBuf::from_str(&expanded).unwrap());
        Ok(path_clean::clean(connected))
    }

    expand_and_clean(s).map_err(|err| err.to_string())
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
    /// Dry-run. Do not create/remove any symlinks.
    #[arg(short = None, long, default_value_t = false)]
    pub dry_run: bool,
    /// Ignore file names by passing a regex to this flag. It can be passed multiple times.
    #[arg(short, long = "ignore")]
    pub ignore_pats: Vec<Regex>,
}

#[derive(Debug, Parser, Clone)]
#[command(about = None, long_about = None)]
pub struct BoxUnboxRcArgs {
    /// Target directory where the symlinks are stored. Must be a directory.
    #[arg(short, long, value_parser = parse_and_expand_pathbuf)]
    pub target: Option<PathBuf>,
    /// Include directories.
    #[arg(short = 'd', long)]
    pub include_dirs: Option<bool>,
    /// Ignore file names by passing a regex to this flag. It can be passed multiple times.
    #[arg(short, long = "ignore")]
    pub ignore_pats: Vec<Regex>,
}

impl BoxUnboxArgs {
    /// Merge these arguments with [`BoxUnboxRcArgs`] parsed from a `.unboxrc` file. This mutates
    /// the current arguments by overriding them with the passed arguments.
    ///
    /// # Arguments
    ///
    /// - `rc_args` - Arguments parsed from RC file.
    pub fn merge_with_rc(&mut self, rc_args: BoxUnboxRcArgs) {
        self.target = rc_args.target.unwrap_or(self.target.clone());
        self.include_dirs = rc_args.include_dirs.unwrap_or(self.include_dirs);

        self.ignore_pats.extend(rc_args.ignore_pats);
    }
}
