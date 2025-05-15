use std::{fmt::Display, path::PathBuf, sync::LazyLock};

use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, de::Error};

use crate::{constants::BASE_DIRS, utils::expand_into_pathbuf};

pub mod error;

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
/// Regex for the `.unboxrc.ron` file, `git` files, and some `.md` files.
fn __ignore_pats_default() -> Vec<Regex> {
    static DEFAULT_REGEX_VEC: LazyLock<Vec<Regex>> = LazyLock::new(|| {
        vec![
            Regex::new(r"\.unboxrc.*$").unwrap(),
            Regex::new(r"^\.git.*$").unwrap(),
            Regex::new(r"^(README|LICEN[CS]E|COPYING).*$").unwrap(),
        ]
    });

    DEFAULT_REGEX_VEC.clone()
}

/// Describes what to do if a target link already exists.
pub enum ExistingFileStrategy {
    /// Throw an error.
    ThrowError,
    /// Ignore the link and continue.
    Ignore,
    /// Move the target link to `<target>.bak`.
    Move,
    /// Overwrite the target with the package version. (destructive!)
    Overwrite,
    /// Overwrite the package with the target version. (destructive!)
    Adopt,
}

/// Describes what type of link to create.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum LinkType {
    SymlinkAbsolute,
    SymlinkRelative,
    HardLink,
}

impl Default for LinkType {
    fn default() -> Self {
        Self::SymlinkAbsolute
    }
}

impl Display for LinkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            LinkType::SymlinkAbsolute => "absolute symlink",
            LinkType::SymlinkRelative => "relative symlink",
            LinkType::HardLink => "hard link",
        };
        write!(f, "{s}")
    }
}

/// A package configuration. Can de/serialize with [`serde`].
#[derive(Clone, Debug, Deserialize, Serialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct PackageConfig {
    /// The path of the package this config is for. This is also the directory where the config
    /// file is located.
    #[serde(skip)]
    pub package: PathBuf,

    /// The target directory.
    #[serde(default = "__target_default", deserialize_with = "__de_pathbuf")]
    pub target: PathBuf,
    /// [`Regex`]'s that determine which file names to ignore.
    #[serde(default = "__ignore_pats_default", with = "serde_regex")]
    pub ignore_pats: Vec<Regex>,
    /// Only link the root package folder, creating one link.
    #[serde(default = "bool::default")]
    pub link_root: bool,
    /// Do not create directories in `target`. If one does not exist, an error is thrown.
    #[serde(default = "bool::default")]
    pub no_create_dirs: bool,
    /// What type of link to create.
    #[serde(default = "LinkType::default")]
    pub link_type: LinkType,
}

impl PackageConfig {
    /// File name this struct will serialize to by default.
    const fn __serde_file_name() -> &'static str {
        ".bub.toml"
    }

    /// Create a new [`PackageConfig`] from the given `package` and `target` paths and default
    /// values for everything else.
    ///
    /// # Arguments
    ///
    /// - `package` - The package directory to create a config for.
    /// - `target` - The target directory of the new config.
    pub fn new<P: Into<PathBuf>, Q: Into<PathBuf>>(package: P, target: Q) -> Self {
        Self {
            package: package.into(),
            target: target.into(),
            ignore_pats: __ignore_pats_default(),
            link_root: bool::default(),
            no_create_dirs: bool::default(),
            link_type: LinkType::default(),
        }
    }

    /// Try to read a config file from the given `package` directory.
    ///
    /// # Arguments
    ///
    /// - `package` - Directory to read from
    ///
    /// # Errors
    ///
    /// An error will be returned if the config file does not exist, cannot be read, or contains
    /// malformed TOML data.
    #[inline]
    pub fn try_from_package<P: Into<PathBuf>>(package: P) -> Result<Self, error::Error> {
        Self::try_from(package.into())
    }
}

impl TryFrom<PathBuf> for PackageConfig {
    type Error = error::Error;

    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        let config_path = value.join(Self::__serde_file_name());

        let config_str =
            &std::fs::read_to_string(&config_path).map_err(|err| error::Error::IoError {
                source: err,
                path: config_path,
            })?;
        let mut parsed_config: Self = toml::from_str(config_str)?;
        parsed_config.package = value;

        Ok(parsed_config)
    }
}
