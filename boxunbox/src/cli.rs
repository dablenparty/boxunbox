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
    /// Always display color (i.e. force it).
    Always,
    /// Automatically determine if color should be used or not.
    Auto,
    /// Never display color.
    Never,
}

/// Describes what to do if a target link already exists.
#[derive(Copy, Clone, Debug, ValueEnum)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum ExistingFileStrategy {
    /// "Adopt" the target file by overwriting the packages file with the target file and
    /// placing a symlink in the target. (destructive!)
    Adopt,
    /// Ignore the link and continue.
    Ignore,
    /// Move the target file to `<target>.bak` and create a symlink at the original target.
    Move,
    /// Overwrite the target file with the package file. (destructive!)
    Overwrite,
    /// Throw an error.
    #[value(name = "error")]
    ThrowError,
}

// FIXME: create man pages with clap_mangen
/// boxunbox is a symlinker inspired by GNU stow.
#[derive(Clone, Debug, Parser)]
#[command(about, long_about = None, styles=__cli_styles(), version)]
#[allow(clippy::struct_excessive_bools)]
pub struct BoxUnboxCli {
    /// Package (directory) to unbox. Specify multiple directories to unbox multiple.
    #[arg(required = true, value_parser = cli_parse_pathbuf, value_hint = ValueHint::DirPath)]
    pub packages: Vec<PathBuf>,

    /// When to show color.
    #[arg(long = "color", default_value_t = ColorOverride::default(), value_name = "WHEN")]
    pub color_override: ColorOverride,
    /// Dry run; show the unboxing plan, but do not execute it.
    #[arg(short = 'd', long)]
    pub dry_run: bool,
    /// Ignore file names with a regex. May be specified multiple times.
    ///
    /// Regex (regular expression) patterns are different from glob patterns. See regex(7) for
    /// an explanation of syntax and <https://regex101.com/> for testing regex patterns.
    #[arg(short, long = "ignore", value_name = "REGEX")]
    pub ignore_pats: Vec<Regex>,
    /// What to do if a file already exists in the target. This has no effect on symlinks that are
    /// created successfully.
    #[arg(short = 'e', long = "if_target_exists", default_value_t = ExistingFileStrategy::default(), value_name = "STRATEGY")]
    pub existing_file_strategy: ExistingFileStrategy,
    /// Create only one link by linking the package directory itself directly to the target.
    ///
    /// For example, when this is `true`, `/path/to/target` would be a symlink pointing to
    /// the `/path/to/package` directory.
    #[arg(short = 'r', long)]
    pub link_root: bool,
    /// Type of link to create.
    #[arg(short, long, value_name = "TYPE")]
    pub link_type: Option<LinkType>,
    /// Save the current CLI parameters to a config file. WARNING: overwrites any existing file!
    ///
    /// When specified in conjunction with `--save-os-config`, both options are respected and two
    /// configs are saved: a generic config AND an OS-specific config. The configs will be
    /// identical.
    #[arg(short = 's', long)]
    pub save_config: bool,
    /// Save the CLI parameters to an OS-specific config insetad of a generic one.
    ///
    /// The OS marker is automatically at compile time. A list of all possible OS values is
    /// available in the Rust docs: <https://doc.rust-lang.org/std/env/consts/constant.OS.html>
    #[arg(short = 'o', long)]
    pub save_os_config: bool,
    /// Directory to unbox the package(s) to. If `--link-root` is enabled, this is where the
    /// symlink will be created. [default: ~]
    #[arg(short, long, value_parser = cli_parse_pathbuf, value_hint = ValueHint::DirPath)]
    pub target: Option<PathBuf>,

    /// Do not create directories at target locations.
    #[cfg(debug_assertions)]
    #[arg(long)]
    pub no_create_dirs: bool,
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
            save_config: false,
            save_os_config: false,
            target: None,

            #[cfg(debug_assertions)]
            no_create_dirs: false,
        }
    }
}
