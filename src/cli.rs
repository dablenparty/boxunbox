use std::path::PathBuf;

use clap::{Parser, ValueHint};
use regex::Regex;

use crate::utils::expand_into_pathbuf;

/**
Parses a `&str` slice as a [`PathBuf`], expand `~` and environment variables and clean the path.

# Arguments

- `s` - `&str` slice.
*/
fn cli_parse_pathbuf(s: &str) -> Result<PathBuf, String> {
    expand_into_pathbuf(s).map_err(|err| err.to_string())
}

/// TODO: main documentation.
#[derive(Debug, Parser)]
pub struct BoxUnboxCli {
    /// Package (directory) to unbox.
    #[arg(required = true, value_parser = cli_parse_pathbuf, value_hint = ValueHint::DirPath)]
    pub package: PathBuf,
    /// Directory to unbox PACKAGE to. [default: ~]
    #[arg(short, long, value_parser = cli_parse_pathbuf, value_hint = ValueHint::DirPath)]
    pub target: Option<PathBuf>,
    /// Ignore file names via regex. May be specified multiple times.
    #[arg(short, long = "ignore")]
    pub ignore_pats: Vec<Regex>,

    /// Dry run; show the unboxing plan, but do not execute it.
    #[arg(long)]
    pub dry_run: bool,
    /// Save the current CLI parameters to the config file. WARNING: overwrites the existing file!
    #[arg(long)]
    pub save_config: bool,

    /// Box a package back up by removing all target symlinks. Works for partially unboxed
    /// packages.
    #[arg(short = 'B', long = "box")]
    pub perform_box: bool,
}
