use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::LazyLock,
};

use anyhow::Context;
use errors::{ParseError, WriteError};
use regex::Regex;
use ron::ser::PrettyConfig;
use serde::{de::Error, Deserialize, Deserializer, Serialize};

use crate::{cli::BoxUnboxCli, constants::BASE_DIRS, utils::expand_into_pathbuf};

pub mod errors;
pub mod plan;

/// Utility function to deserialize a [`PathBuf`] while expanding environment variables and `~`.
///
/// # Arguments
///
/// - `d` - Argument to deserialize, expected to be `&str`.
fn __de_pathbuf<'de, D>(d: D) -> Result<PathBuf, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(d)?;
    expand_into_pathbuf(s).map_err(D::Error::custom)
}

/// Utility function returning the default value for [`PackageConfig::target`], which is the users
/// home directory.
fn __target_default() -> PathBuf {
    BASE_DIRS.home_dir().to_path_buf()
}

/// Utility function returning the default value for [`PackageConfig::ignore_pats`], which is a
/// Regex for the `.unboxrc.ron` file.
fn __ignore_pats_default() -> Vec<Regex> {
    static RC_REGEX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new("\\.unboxrc(\\.ron)?$").unwrap());

    vec![RC_REGEX.clone()]
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PackageConfig {
    #[serde(skip)]
    pub package: PathBuf,
    #[serde(skip)]
    pub dry_run: bool,

    #[serde(default = "__target_default", deserialize_with = "__de_pathbuf")]
    pub target: PathBuf,
    #[serde(default = "__ignore_pats_default", with = "serde_regex")]
    pub ignore_pats: Vec<Regex>,
}

impl TryFrom<BoxUnboxCli> for PackageConfig {
    type Error = io::Error;

    fn try_from(value: BoxUnboxCli) -> Result<Self, Self::Error> {
        let BoxUnboxCli {
            package,
            target,
            ignore_pats: cli_ignore_pats,
            dry_run,
            ..
        } = value;

        // prepend default ignore pattern(s)
        let mut ignore_pats = __ignore_pats_default();
        ignore_pats.extend(cli_ignore_pats);

        let conf = Self {
            package: package.canonicalize()?,
            target: target.unwrap_or_else(__target_default).canonicalize()?,
            ignore_pats,
            dry_run,
        };
        Ok(conf)
    }
}

impl PackageConfig {
    /// Expected file name of the RC file.
    const fn __rc_file_name() -> &'static str {
        // TODO: consider allowing multiple names
        ".unboxrc.ron"
    }

    /// Merge with [`BoxUnboxCli`] args. Consumes this struct.
    ///
    /// # Arguments
    ///
    /// - `cli` - CLI args to merge with.
    pub fn merge_with_cli(self, cli: &BoxUnboxCli) -> io::Result<Self> {
        let mut ignore_pats = self.ignore_pats;
        ignore_pats.extend(cli.ignore_pats.clone());

        let conf = Self {
            package: self.package,
            target: cli.target.clone().map_or(Ok(self.target), |p| {
                if p.is_relative() {
                    p.canonicalize()
                } else {
                    Ok(p)
                }
            })?,
            dry_run: cli.dry_run,
            ignore_pats,
        };

        Ok(conf)
    }

    /// Try to parse [`PackageConfig`] from a given package path.
    ///
    /// # Arguments
    ///
    /// - `package` - Package [`Path`].
    ///
    /// # Errors
    ///
    /// An error is returned if:
    ///
    /// - The RC file doesn't exist.
    /// - Failure to read RC file.
    /// - Failure to parse RC file with [`ron`].
    pub fn try_from_package<P: AsRef<Path>>(package: P) -> Result<Self, ParseError> {
        let package = package.as_ref();
        let rc_file = package.join(PackageConfig::__rc_file_name());

        if !rc_file.exists() {
            return Err(ParseError::FileNotFound(rc_file));
        }

        let rc_str = fs::read_to_string(&rc_file)
            .with_context(|| format!("failed to read file: {rc_file:?}"))?;

        let mut rc: PackageConfig = ron::from_str(&rc_str)?;
        rc.package = package.to_path_buf();

        Ok(rc)
    }

    /// Save this [`PackageConfig`] to a `package` directory.
    ///
    /// # Arguments
    ///
    /// - `package` - Package to save this config to.
    ///
    /// # Errors
    ///
    /// An error is returned if:
    ///
    /// - This struct fails to serialize into RON.
    /// - The file cannot be created/written to
    pub fn save_to_package<P: AsRef<Path>>(&self, package: P) -> Result<(), WriteError> {
        let mut clone_self = self.clone();
        let home_dir = BASE_DIRS.home_dir();

        if let Ok(path) = clone_self.target.strip_prefix(home_dir) {
            clone_self.target = PathBuf::from("~/").join(path);
        }

        let package = package.as_ref();
        let rc_file = package.join(PackageConfig::__rc_file_name());

        // TODO: do something if the config already exists, maybe an error?
        // WARN: this overwrites the existing file, be careful!
        let ron_str =
            ron::ser::to_string_pretty(&clone_self, PrettyConfig::new().struct_names(true))?;
        fs::write(rc_file, ron_str)?;

        Ok(())
    }
}
