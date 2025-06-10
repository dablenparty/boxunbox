use std::{
    fmt::{self, Display},
    path::PathBuf,
};

use anyhow::Context;
use clap::{
    Parser, ValueEnum, ValueHint,
    builder::{Styles, styling::AnsiColor},
};
use regex::Regex;

use crate::{package::LinkType, utils::expand_into_pathbuf};

/// Get the color styles for the CLI help menu.
fn __cli_styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Yellow.on_default())
        .usage(AnsiColor::Yellow.on_default())
        .literal(AnsiColor::Green.on_default())
        .placeholder(AnsiColor::Green.on_default())
}

/// Parses a `&str` slice as a [`PathBuf`], expand `~` and environment variables and clean the path.
///
/// # Arguments
///
/// - `s` - `&str` slice.
fn cli_parse_pathbuf(s: &str) -> Result<PathBuf, String> {
    expand_into_pathbuf(s)
        .and_then(|p| {
            dunce::canonicalize(&p)
                .with_context(|| format!("failed to canonicalize {}", p.display()))
        })
        .map_err(|err| err.to_string())
}

/// Override the color setting. Default is [`ColorOverride::Auto`].
#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum ColorOverride {
    Always,
    Auto,
    Never,
}

/// Describes what to do if a target link already exists.
#[derive(Copy, Clone, Debug, ValueEnum)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum ExistingFileStrategy {
    /// Overwrite the package with the target version. (destructive!)
    Adopt,
    /// Ignore the link and continue.
    Ignore,
    /// Move the target link to `<target>.bak`.
    Move,
    /// Overwrite the target with the package version. (destructive!)
    Overwrite,
    /// Throw an error.
    #[value(name = "error")]
    ThrowError,
}

/// boxunbox is a symlinker inspired by GNU stow.
#[derive(Clone, Debug, Parser)]
#[command(about, long_about = None, styles=__cli_styles(), version)]
#[allow(clippy::struct_excessive_bools)]
pub struct BoxUnboxCli {
    /// Package (directory) to unbox. Specify multiple to unbox multiple.
    #[arg(required = true, value_parser = cli_parse_pathbuf, value_hint = ValueHint::DirPath)]
    pub packages: Vec<PathBuf>,

    /// When to show color.
    #[arg(long = "color", default_value_t = ColorOverride::default())]
    pub color_override: ColorOverride,
    /// Dry run; show the unboxing plan, but do not execute it.
    #[arg(short = 'd', long)]
    pub dry_run: bool,
    /// Ignore file names via regex. May be specified multiple times.
    #[arg(short, long = "ignore")]
    pub ignore_pats: Vec<Regex>,
    #[arg(short = 'e', long = "if_target_exists", default_value_t = ExistingFileStrategy::default())]
    pub existing_file_strategy: ExistingFileStrategy,
    /// Link the package directory itself.
    #[arg(short = 'r', long)]
    pub link_root: bool,
    /// Type of symbolic (or hard) link to create.
    #[arg(short, long)]
    pub link_type: Option<LinkType>,
    /// Do not create directories at target locations.
    #[arg(long)]
    pub no_create_dirs: bool,
    /// Save the current CLI parameters to the config file. WARNING: overwrites the existing file!
    #[arg(short = 's', long)]
    pub save_config: bool,
    /// Save an OS-specific config insetad of a generic one. Overwrites --save-config.
    #[arg(short = 'o', long)]
    pub save_os_config: bool,
    /// Directory to unbox PACKAGE to. [default: ~]
    #[arg(short, long, value_parser = cli_parse_pathbuf, value_hint = ValueHint::DirPath)]
    pub target: Option<PathBuf>,
}

impl Default for ColorOverride {
    fn default() -> Self {
        Self::Auto
    }
}

impl Default for ExistingFileStrategy {
    fn default() -> Self {
        Self::ThrowError
    }
}

impl Display for ColorOverride {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ColorOverride::Always => "always",
            ColorOverride::Auto => "auto",
            ColorOverride::Never => "never",
        };

        write!(f, "{s}")
    }
}

impl Display for ExistingFileStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ExistingFileStrategy::ThrowError => "error",
            ExistingFileStrategy::Ignore => "ignore",
            ExistingFileStrategy::Move => "move",
            ExistingFileStrategy::Overwrite => "overwrite",
            ExistingFileStrategy::Adopt => "adopt",
        };

        write!(f, "{s}")
    }
}

#[cfg(test)]
impl BoxUnboxCli {
    pub(crate) fn new<P: Into<PathBuf>>(package: P) -> Self {
        Self {
            packages: vec![package.into()],
            color_override: ColorOverride::default(),
            dry_run: false,
            existing_file_strategy: ExistingFileStrategy::default(),
            ignore_pats: Vec::new(),
            link_root: false,
            link_type: None,
            no_create_dirs: false,
            save_config: false,
            save_os_config: false,
            target: None,
        }
    }
}
