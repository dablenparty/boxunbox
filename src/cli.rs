use std::{fmt, path::PathBuf};

use clap::{Parser, ValueEnum, ValueHint};
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

/// Override the color setting. Default is [`ColorOverride::Auto`].
#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum ColorOverride {
    Always,
    Auto,
    Never,
}

impl Default for ColorOverride {
    fn default() -> Self {
        Self::Auto
    }
}

impl fmt::Display for ColorOverride {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ColorOverride::Always => "always",
            ColorOverride::Auto => "auto",
            ColorOverride::Never => "never",
        };

        write!(f, "{s}")
    }
}

/// boxunbox (or just unbox) is a symlinker inspired by GNU stow.
#[derive(Clone, Debug, Parser)]
#[command(about, long_about = None, version)]
#[allow(clippy::struct_excessive_bools)]
pub struct BoxUnboxCli {
    /// Package (directory) to unbox. Specify multiple to unbox multiple.
    #[arg(required = true, value_parser = cli_parse_pathbuf, value_hint = ValueHint::DirPath)]
    pub packages: Vec<PathBuf>,
    /// Directory to unbox PACKAGE to. [default: ~]
    #[arg(short, long, value_parser = cli_parse_pathbuf, value_hint = ValueHint::DirPath)]
    pub target: Option<PathBuf>,
    /// Ignore file names via regex. May be specified multiple times.
    #[arg(short, long = "ignore")]
    pub ignore_pats: Vec<Regex>,
    /// Link the package directory directly to the target.
    #[arg(short, long)]
    pub link_root: bool,
    /// Create relative links instead of absolute links.
    #[arg(short = 'r', long = "relative-links")]
    pub use_relative_links: bool,
    /// Create hard links instead of soft links (symlinks)
    #[arg(short = 'H', long = "hard-links")]
    pub use_hard_links: bool,
    /// Do not create directories at target locations. Only applies to strategies that create
    /// directories.
    #[arg(long)]
    pub no_create_dirs: bool,

    /// Dry run; show the unboxing plan, but do not execute it.
    #[arg(short = 'd', long)]
    pub dry_run: bool,
    /// Force the creation of symlinks. Overwrites existing files.
    #[arg(short, long)]
    pub force: bool,
    /// Ignore "target exists" errors.
    #[arg(short = 'e', long)]
    pub ignore_exists: bool,
    /// Save the current CLI parameters to the config file. WARNING: overwrites the existing file!
    #[arg(short = 's', long)]
    pub save_config: bool,
    /// Save an OS-specific config insetad of a generic one. Overwrites --save-config.
    #[arg(short = 'o', long)]
    pub save_os_config: bool,

    /// Box a package back up by removing all target symlinks. Works for partially unboxed
    /// packages.
    #[arg(short = 'B', long = "box")]
    pub perform_box: bool,
    /// When to show color.
    #[arg(long = "color", default_value_t = ColorOverride::default())]
    pub color_override: ColorOverride,
}
