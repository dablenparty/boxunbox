use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::LazyLock,
};

use anyhow::Context;
use const_format::formatc;
use errors::{ParseError, WriteError};
use regex::Regex;
use ron::ser::PrettyConfig;
use serde::{Deserialize, Deserializer, Serialize, de::Error};

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

#[derive(Clone, Debug, Deserialize, Serialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct PackageConfig {
    #[serde(skip)]
    pub package: PathBuf,
    #[serde(skip)]
    pub force: bool,
    #[serde(skip)]
    pub ignore_exists: bool,
    #[serde(skip)]
    pub perform_box: bool,

    #[serde(default = "__target_default", deserialize_with = "__de_pathbuf")]
    pub target: PathBuf,
    #[serde(default = "__ignore_pats_default", with = "serde_regex")]
    pub ignore_pats: Vec<Regex>,
    #[serde(default = "bool::default")]
    pub link_root: bool,
    #[serde(default = "bool::default")]
    pub no_create_dirs: bool,
    #[serde(default = "bool::default")]
    pub use_relative_links: bool,
    #[serde(default = "bool::default")]
    pub use_hard_links: bool,
}

impl TryFrom<PathBuf> for PackageConfig {
    type Error = ParseError;

    fn try_from(package: PathBuf) -> Result<Self, Self::Error> {
        let default_rc_path = package.join(PackageConfig::__rc_file_name());
        let os_rc_path = package.join(PackageConfig::__os_rc_file_name());

        let rc_file = if os_rc_path.try_exists().unwrap_or(false) {
            os_rc_path
        } else if default_rc_path.try_exists().unwrap_or(false) {
            default_rc_path
        } else {
            // no config found for this package
            return Err(ParseError::ConfigNotFound(package));
        };

        #[cfg(debug_assertions)]
        println!("reading config: {rc_file:?}");

        let rc_str = fs::read_to_string(&rc_file)
            .with_context(|| format!("failed to read file: {rc_file:?}"))?;

        let mut rc: PackageConfig = ron::from_str(&rc_str)?;
        rc.package = package;

        Ok(rc)
    }
}

impl PackageConfig {
    /// Expected file name of the RC file.
    const fn __rc_file_name() -> &'static str {
        // TODO: consider allowing multiple names
        ".unboxrc.ron"
    }

    /// Expected file name of the OS-specific RC file.
    const fn __os_rc_file_name() -> &'static str {
        formatc!(".unboxrc.{}.ron", std::env::consts::OS)
    }

    /// Create a new [`PackageConfig`] for the given [`package`] with default values.
    ///
    /// # Arguments
    ///
    /// - `package` - Package to make config for.
    pub fn new<P: Into<PathBuf>>(package: P) -> Self {
        Self {
            package: package.into(),
            force: false,
            ignore_exists: false,
            perform_box: false,
            target: __target_default(),
            ignore_pats: __ignore_pats_default(),
            link_root: false,
            no_create_dirs: false,
            use_relative_links: false,
            use_hard_links: false,
        }
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
            force: cli.force || self.force,
            ignore_exists: cli.ignore_exists || self.ignore_exists,
            ignore_pats,
            link_root: cli.link_root || self.link_root,
            no_create_dirs: cli.no_create_dirs || self.no_create_dirs,
            perform_box: cli.perform_box || self.perform_box,
            use_relative_links: self.use_relative_links || cli.use_relative_links,
            use_hard_links: self.use_hard_links || cli.use_hard_links,
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
    #[inline]
    pub fn try_from_package<P: Into<PathBuf>>(package: P) -> Result<Self, ParseError> {
        Self::try_from(package.into())
    }

    /// Save this [`PackageConfig`] to a `package` directory.
    ///
    /// # Arguments
    ///
    /// - `package` - Package to save this config to.
    /// - `use_os` - Save as an OS-specific config. See [`std::env::consts::OS`].
    ///
    /// # Errors
    ///
    /// An error is returned if:
    ///
    /// - This struct fails to serialize into RON.
    /// - The file cannot be created/written to
    pub fn save_to_package<P: AsRef<Path>>(
        &self,
        package: P,
        use_os: bool,
    ) -> Result<(), WriteError> {
        let mut clone_self = self.clone();
        let home_dir = BASE_DIRS.home_dir();

        if let Ok(path) = clone_self.target.strip_prefix(home_dir) {
            clone_self.target = PathBuf::from("~/").join(path);
        }

        let package = package.as_ref();
        let rc_file = if use_os {
            package.join(PackageConfig::__os_rc_file_name())
        } else {
            package.join(PackageConfig::__rc_file_name())
        };

        #[cfg(debug_assertions)]
        println!("saving config to {rc_file:?}");

        // WARN: this overwrites the existing file, be careful!
        let ron_str =
            ron::ser::to_string_pretty(&clone_self, PrettyConfig::new().struct_names(true))?;
        fs::write(rc_file, ron_str)?;

        Ok(())
    }
}
