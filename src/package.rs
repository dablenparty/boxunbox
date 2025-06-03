use std::{fmt::Display, path::PathBuf, sync::LazyLock};

use clap::ValueEnum;
use const_format::formatc;
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, de::Error};

use crate::{cli::BoxUnboxCli, constants::BASE_DIRS, utils::expand_into_pathbuf};

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
    // NOTE: don't use &str or deserializing will fail for strings
    let s: String = Deserialize::deserialize(d)?;
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
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
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

impl PartialEq for PackageConfig {
    fn eq(&self, other: &Self) -> bool {
        self.package == other.package
            && self.target == other.target
            && self.ignore_pats.len() == other.ignore_pats.len()
            && self
                .ignore_pats
                .iter()
                .zip(&other.ignore_pats)
                .all(|(l, r)| l.as_str() == r.as_str())
            && self.link_root == other.link_root
            && self.link_type == other.link_type
    }
}

impl Eq for PackageConfig {}

/// A package configuration. Can de/serialize with [`serde`].
#[derive(Clone, Debug, Deserialize, Serialize)]
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
    /// What type of link to create.
    #[serde(default = "LinkType::default")]
    pub link_type: LinkType,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[warn(deprecated_in_future)]
#[allow(clippy::struct_excessive_bools)]
pub struct OldPackageConfig {
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
            link_type: LinkType::default(),
        }
    }

    /// Try to read a config file from the given `package` directory. Package options are
    /// merged with [`BoxUnboxCli`] flags of the same name.
    ///
    /// # Arguments
    ///
    /// - `package` - Directory to read from
    /// - `cli` - CLI options
    ///
    /// # Errors
    ///
    /// An error will be returned if the config file does not exist, cannot be read, or contains
    /// malformed TOML data.
    pub fn try_from_package<P: Into<PathBuf>>(
        package: P,
        cli: &BoxUnboxCli,
    ) -> Result<Self, error::ConfigRead> {
        // TODO: if old config exists (.unboxrc.ron), replace it with a toml file
        let package = package.into();
        let toml_path = package.join(Self::__serde_file_name());
        let mut config = Self::try_from(toml_path)?;
        config.ignore_pats.extend_from_slice(&cli.ignore_pats[..]);
        config.link_root |= cli.link_root;
        if let Some(link_type) = cli.link_type {
            config.link_type = link_type;
        }
        if let Some(target) = cli.target.as_ref() {
            config.target.clone_from(target);
        }

        Ok(config)
    }

    #[warn(deprecated_in_future)]
    pub fn from_old_package<P: Into<PathBuf>>(package: P, value: OldPackageConfig) -> Self {
        Self {
            package: package.into(),
            target: value.target,
            ignore_pats: value.ignore_pats,
            link_root: value.link_root,
            link_type: match (value.use_relative_links, value.use_hard_links) {
                (_, true) => LinkType::HardLink,
                (false, false) => LinkType::SymlinkAbsolute,
                (true, false) => LinkType::SymlinkRelative,
            },
        }
    }

    /// Get the disk path for this `PackageConfig`.
    #[inline]
    fn disk_path(&self) -> PathBuf {
        self.package.join(PackageConfig::__serde_file_name())
    }

    /// Save this `PackageConfig` to a package directory.
    ///
    /// # Errors
    ///
    /// An error will be returned if the config fails to serialize or the file cannot be
    /// written to for some reason.
    pub fn save_to_package(&self) -> Result<(), error::ConfigWrite> {
        let config_path = self.disk_path();
        let config_str = toml::to_string_pretty(self)?;
        // WARN: this truncates the existing file. be careful!
        std::fs::write(&config_path, config_str).map_err(|err| error::ConfigWrite::Io {
            source: err,
            path: config_path,
        })?;
        Ok(())
    }
}

impl TryFrom<PathBuf> for PackageConfig {
    type Error = error::ConfigRead;

    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        let config_path = value;

        if !config_path
            .try_exists()
            .map_err(|err| error::ConfigRead::Io {
                source: err,
                path: config_path.clone(),
            })?
        {
            return Err(error::ConfigRead::FileNotFound(config_path));
        }

        let config_str =
            &std::fs::read_to_string(&config_path).map_err(|err| error::ConfigRead::Io {
                source: err,
                path: config_path.clone(),
            })?;
        let mut parsed_config: Self = toml::from_str(config_str)?;
        parsed_config.package = config_path
            .parent()
            .unwrap_or_else(|| panic!("file '{}' has no parent", config_path.display()))
            .to_path_buf();

        Ok(parsed_config)
    }
}

impl OldPackageConfig {
    /// Expected file name of the RC file.
    const fn __rc_file_name() -> &'static str {
        // TODO: consider allowing multiple names
        ".unboxrc.ron"
    }

    /// Expected file name of the OS-specific RC file.
    const fn __os_rc_file_name() -> &'static str {
        formatc!(".unboxrc.{}.ron", std::env::consts::OS)
    }
}

#[cfg(test)]
impl Default for OldPackageConfig {
    fn default() -> Self {
        Self {
            target: __target_default(),
            ignore_pats: __ignore_pats_default(),
            link_root: false,
            no_create_dirs: false,
            use_relative_links: false,
            use_hard_links: false,
        }
    }
}

impl TryFrom<PathBuf> for OldPackageConfig {
    type Error = error::ConfigRead;

    fn try_from(package: PathBuf) -> Result<Self, Self::Error> {
        let default_rc_path = package.join(OldPackageConfig::__rc_file_name());
        let os_rc_path = package.join(OldPackageConfig::__os_rc_file_name());

        let rc_file = if os_rc_path.try_exists().unwrap_or(false) {
            os_rc_path
        } else if default_rc_path.try_exists().unwrap_or(false) {
            default_rc_path
        } else {
            // no config found for this package
            return Err(error::ConfigRead::FileNotFound(package));
        };

        #[cfg(debug_assertions)]
        println!("reading config: {}", rc_file.display());

        let rc_str = std::fs::read_to_string(&rc_file).map_err(|err| error::ConfigRead::Io {
            source: err,
            path: rc_file.clone(),
        })?;

        let rc: OldPackageConfig = ron::from_str(&rc_str)?;

        Ok(rc)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Context;

    use crate::test_utils::make_tmp_tree;

    use super::*;

    #[test]
    fn test_try_from_package() -> anyhow::Result<()> {
        let package = make_tmp_tree().context("failed to make test package")?;
        let package_path = package.path();
        let cli = BoxUnboxCli::new(package_path);

        let conf = PackageConfig::try_from_package(package_path, &cli)
            .context("failed to create package config from package")?;

        assert_eq!(conf.package, package_path);
        assert_eq!(conf.target, BASE_DIRS.home_dir());
        let expected_ignore_pats = __ignore_pats_default();
        assert!(
            conf.ignore_pats.len() == expected_ignore_pats.len()
                && conf
                    .ignore_pats
                    .iter()
                    .zip(expected_ignore_pats)
                    .all(|(a, b)| a.as_str() == b.as_str())
        );
        assert!(!conf.link_root);
        assert_eq!(conf.link_type, LinkType::SymlinkAbsolute);

        Ok(())
    }

    #[test]
    fn test_try_from_package_respects_cli() -> anyhow::Result<()> {
        let package = make_tmp_tree().context("failed to make test package")?;
        let package_path = package.path();
        let mut cli = BoxUnboxCli::new(package_path);
        // change EVERY value from the default for a comprehensive test
        cli.link_root = true;
        cli.link_type = Some(LinkType::HardLink);
        let test_regex = Regex::new("^test$").context("failed to compile test Regex")?;
        cli.ignore_pats = vec![test_regex];
        let expected_target = PathBuf::from("/path/to/test/target");
        cli.target = Some(expected_target.clone());

        let conf = PackageConfig::try_from_package(package_path, &cli)
            .context("failed to create package config from package")?;

        assert_eq!(conf.package, package_path);
        assert_eq!(conf.target, expected_target);
        let expected_ignore_pats = __ignore_pats_default()
            .into_iter()
            .chain(cli.ignore_pats.clone())
            .collect::<Vec<Regex>>();
        assert!(
            conf.ignore_pats.len() == expected_ignore_pats.len()
                && conf
                    .ignore_pats
                    .iter()
                    .zip(expected_ignore_pats)
                    .all(|(a, b)| a.as_str() == b.as_str())
        );
        assert!(conf.link_root);
        assert_eq!(conf.link_type, LinkType::HardLink);

        Ok(())
    }

    #[test]
    fn test_save_to_package() -> anyhow::Result<()> {
        let package = tempfile::tempdir().context("failed to make test package")?;
        let conf = PackageConfig::new(package.path(), BASE_DIRS.home_dir());
        conf.save_to_package()
            .context("failed to save config to test package")?;
        let conf_path = package.path().join(PackageConfig::__serde_file_name());
        let expected_conf_str =
            toml::to_string_pretty(&conf).context("failed to serialize test config")?;
        let actual_conf_str =
            std::fs::read_to_string(&conf_path).context("failed to read test config")?;

        assert!(
            conf_path
                .try_exists()
                .context("failed to verify existence of test config")?,
            "test config file could not be found"
        );
        assert_eq!(
            expected_conf_str, actual_conf_str,
            "contents of test config file do not match serialized test config"
        );

        Ok(())
    }

    #[test]
    fn test_from_old_package() -> anyhow::Result<()> {
        let package = make_tmp_tree().context("failed to make test package")?;
        let package_path = package.path();
        let old_config = OldPackageConfig::default();
        let expected = PackageConfig::new(package_path, __target_default());
        let actual = PackageConfig::from_old_package(package_path, old_config);

        assert_eq!(expected, actual);

        Ok(())
    }
}
