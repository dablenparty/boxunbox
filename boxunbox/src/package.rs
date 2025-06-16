use std::{
    fmt::Display,
    fs,
    path::{Path, PathBuf},
    sync::LazyLock,
};

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
/// - `d` - Argument to deserialize, expected to be `String`.
fn __de_pathbuf<'de, D>(d: D) -> Result<PathBuf, D::Error>
where
    D: Deserializer<'de>,
{
    // NOTE: don't use &str or deserializing will fail for strings
    let s: String = Deserialize::deserialize(d)?;
    expand_into_pathbuf(s).map_err(D::Error::custom)
}

/// Utility function returning the default value for [`PackageConfig::ignore_pats`], which is a
/// Regex for the config file, `git` files, and some `.md` files.
fn __ignore_pats_default() -> Vec<Regex> {
    static DEFAULT_REGEX_VEC: LazyLock<Vec<Regex>> = LazyLock::new(|| {
        vec![
            #[warn(deprecated_in_future)]
            Regex::new(r"\.unboxrc.*$").unwrap(),
            Regex::new(r"\.bub\.toml$").unwrap(),
            Regex::new(r"^\.git.*$").unwrap(),
            Regex::new(r"^(README|LICEN[CS]E|COPYING).*$").unwrap(),
        ]
    });

    DEFAULT_REGEX_VEC.clone()
}

/// Utility function returning the default value for [`PackageConfig::target`], which is the users
/// home directory.
#[cfg(not(test))]
fn __target_default() -> PathBuf {
    BASE_DIRS.home_dir().to_path_buf()
}

/// Utility function returning the default **_test_** value for [`PackageConfig::target`], which is
/// created from [`crate::test_utils::TEST_TARGET`].
#[cfg(test)]
fn __target_default() -> PathBuf {
    PathBuf::from(crate::test_utils::TEST_TARGET)
}

/// Describes what type of link to create.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
pub enum LinkType {
    /// A soft link (symlink) pointing to an absolute path.
    #[value(name = "absolute")]
    SymlinkAbsolute,
    /// A soft link (symlink) pointing to a relative path.
    #[value(name = "relative")]
    SymlinkRelative,
    /// A hard link.
    #[value(name = "hard")]
    HardLink,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename = "PackageConfig")]
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

impl Default for LinkType {
    fn default() -> Self {
        Self::SymlinkAbsolute
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

impl Eq for PackageConfig {}

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

        let rc_str = fs::read_to_string(&rc_file).map_err(|err| error::ConfigRead::Io {
            source: err,
            path: rc_file.clone(),
        })?;

        let rc: OldPackageConfig = ron::from_str(&rc_str)?;

        Ok(rc)
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
            &fs::read_to_string(&config_path).map_err(|err| error::ConfigRead::Io {
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

    #[cfg(test)]
    fn save_to_disk<P: AsRef<Path>>(&self, config_path: P) -> anyhow::Result<()> {
        use ron::ser::PrettyConfig;

        let config_path = config_path.as_ref();

        let ron_str = ron::ser::to_string_pretty(self, PrettyConfig::new().struct_names(true))?;
        fs::write(config_path, ron_str)?;

        Ok(())
    }
}

impl PackageConfig {
    /// File name this struct will serialize to by default.
    const fn __serde_file_name() -> &'static str {
        ".bub.toml"
    }

    /// File name this struct will serialize to when saving to an OS-specific config.
    const fn __serde_os_file_name() -> &'static str {
        formatc!(".bub.{}.toml", std::env::consts::OS)
    }

    /// Create a new [`PackageConfig`] with the given `package` and default values.
    ///
    /// # Arguments
    ///
    /// - `package` - The package this config is for.
    pub fn new<P: Into<PathBuf>>(package: P) -> Self {
        Self {
            package: package.into(),
            target: __target_default(),
            ignore_pats: __ignore_pats_default(),
            link_root: bool::default(),
            link_type: LinkType::default(),
        }
    }

    /// Create a new [`PackageConfig`] from the given `package` and `target` paths and default
    /// values for everything else.
    ///
    /// # Arguments
    ///
    /// - `package` - The package directory to create a config for.
    /// - `target` - The target directory of the new config.
    #[cfg(test)]
    pub fn new_with_target<P: Into<PathBuf>, Q: Into<PathBuf>>(package: P, target: Q) -> Self {
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
    pub fn try_from_package<P: Into<PathBuf>>(package: P) -> Result<Self, error::ConfigRead> {
        let package: PathBuf = package.into();
        // OS-specific configs always take precedence
        let os_toml_path = package.join(Self::__serde_os_file_name());
        let toml_path = if os_toml_path.try_exists().unwrap_or(false) {
            os_toml_path
        } else {
            package.join(Self::__serde_file_name())
        };

        Self::try_from(toml_path)
    }

    pub fn init<P: Into<PathBuf>>(
        package: P,
        cli: &BoxUnboxCli,
    ) -> Result<Self, error::ConfigRead> {
        let package = package.into();
        let mut config = Self::try_from_package(package)?;
        config.merge_with_cli(cli);

        Ok(config)
    }

    pub fn merge_with_cli(&mut self, cli: &BoxUnboxCli) {
        self.ignore_pats.extend_from_slice(&cli.ignore_pats[..]);
        self.link_root |= cli.link_root;
        if let Some(link_type) = cli.link_type {
            self.link_type = link_type;
        }
        if let Some(target) = cli.target.as_ref() {
            self.target.clone_from(target);
        }
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
        self.package.join(Self::__serde_file_name())
    }

    fn os_disk_path(&self) -> PathBuf {
        self.package.join(Self::__serde_os_file_name())
    }

    /// Utility function for saving this [`PackageConfig`] to a given path.
    ///
    /// # Arguments
    ///
    /// - `config_path` - [`Path`] to serialize this config to.
    fn __inner_save_to_package<P: AsRef<Path>>(
        &self,
        config_path: P,
    ) -> Result<(), error::ConfigWrite> {
        let config_path = config_path.as_ref();
        // TODO: replace $HOME in paths with ~
        let config_str = toml::to_string_pretty(self)?;
        // WARN: this truncates the existing file. be careful!
        fs::write(config_path, config_str).map_err(|err| error::ConfigWrite::Io {
            source: err,
            path: config_path.to_path_buf(),
        })?;
        Ok(())
    }

    /// Save this `PackageConfig` to a package directory.
    ///
    /// # Errors
    ///
    /// An error will be returned if the config fails to serialize or the file cannot be
    /// written to for some reason.
    pub fn save_to_package(&self) -> Result<(), error::ConfigWrite> {
        self.__inner_save_to_package(self.disk_path())
    }

    /// Save this `PackageConfig` to a package as an OS-specific config. This uses
    /// [`std::env::consts::OS`] at runtime to determine which system the user is on.
    pub fn save_to_os_package(&self) -> Result<(), error::ConfigWrite> {
        self.__inner_save_to_package(self.os_disk_path())
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Context;

    use crate::test_utils::{TEST_TARGET, make_tmp_tree};

    use super::*;

    #[test]
    fn test_try_from_package() -> anyhow::Result<()> {
        let package = make_tmp_tree().context("failed to make test package")?;
        let package_path = package.path();
        let conf = PackageConfig::try_from_package(package_path)
            .context("failed to create package config from package")?;

        assert_eq!(conf.package, package_path);
        assert_eq!(conf.target, PathBuf::from(TEST_TARGET));
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
    fn test_try_from_os_package() -> anyhow::Result<()> {
        let package = make_tmp_tree().context("failed to make test package")?;
        let package_path = package.path();
        let config_path = package_path.join(PackageConfig::__serde_file_name());
        let expected_config_path = package_path.join(PackageConfig::__serde_os_file_name());
        fs::rename(&config_path, &expected_config_path)
            .context("failed to move test config to OS config")?;
        assert!(
            !config_path.try_exists().with_context(|| format!(
                "failed to verify existence of {}",
                config_path.display()
            ))?,
            "unexpected config file at {}",
            config_path.display()
        );
        assert!(
            expected_config_path.try_exists().with_context(|| format!(
                "failed to verify existence of {}",
                expected_config_path.display()
            ))?,
            "expected config file at {}",
            expected_config_path.display()
        );
        let conf = PackageConfig::try_from_package(package_path)
            .context("failed to create package config from package")?;

        assert_eq!(conf.package, package_path);
        assert_eq!(conf.target, PathBuf::from(TEST_TARGET));
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
    fn test_init() -> anyhow::Result<()> {
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

        let conf = PackageConfig::init(package_path, &cli)
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
        let conf = PackageConfig::new(package.path());
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
    fn test_save_to_os_package() -> anyhow::Result<()> {
        let package = tempfile::tempdir().context("failed to make test package")?;
        let conf = PackageConfig::new(package.path());
        conf.save_to_os_package()
            .context("failed to save config to test package")?;
        let conf_path = package.path().join(PackageConfig::__serde_os_file_name());
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
        let expected = PackageConfig::new(package_path);
        let actual = PackageConfig::from_old_package(package_path, old_config);

        assert_eq!(expected, actual);

        Ok(())
    }

    #[test]
    fn test_from_old_package_file() -> anyhow::Result<()> {
        let package = tempfile::tempdir().context("failed to make test package")?;
        let package_path = package.path();
        let target = tempfile::tempdir().context("failed to make test target")?;
        let target_path = target.path();
        let old_config_path = package_path.join(OldPackageConfig::__rc_file_name());
        let old_config = OldPackageConfig {
            target: target_path.to_path_buf(),
            ..Default::default()
        };
        old_config
            .save_to_disk(old_config_path)
            .context("failed to save test old config")?;

        let actual_old_config = OldPackageConfig::try_from(package_path.to_path_buf())?;
        let expected_config = PackageConfig::new_with_target(package_path, target_path);
        let actual_config = PackageConfig::from_old_package(package_path, actual_old_config);

        assert_eq!(expected_config, actual_config);

        Ok(())
    }
}
