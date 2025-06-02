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
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
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
            && self.no_create_dirs == other.no_create_dirs
            && self.link_type == other.link_type
    }
}

impl Eq for PackageConfig {}

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
    pub fn try_from_package<P: Into<PathBuf>>(package: P) -> Result<Self, error::ConfigRead> {
        // TODO: if old config exists (.unboxrc.ron), replace it with a toml file
        Self::try_from(package.into().join(Self::__serde_file_name()))
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

#[cfg(test)]
mod tests {
    use std::os::unix::ffi::OsStrExt;

    use anyhow::Context;
    use tempfile::TempDir;

    use super::*;

    /// Creates a new temporary file tree for use in integration tests. Each call to this function will
    /// create a _new_ temporary directory; however, every directory will have the same structure:
    ///
    /// ```text
    /// <tempdir>
    /// └── src
    ///     ├── folder1
    ///     │   ├── nested1.txt
    ///     │   └── test_ignore2.txt
    ///     ├── folder2
    ///     │   ├── nested2.txt
    ///     │   └── 'nested2 again.txt'
    ///     ├── test.txt
    ///     └── test_ignore.txt
    /// ```
    fn make_tmp_tree() -> anyhow::Result<TempDir> {
        const FILES_TO_CREATE: [&str; 6] = [
            "folder1/nested1.txt",
            "folder1/test_ignore2.txt",
            "folder2/nested2.txt",
            "folder2/nested2 again.txt",
            "test.txt",
            "test_ignore.txt",
        ];

        let temp_dir = tempfile::tempdir().context("failed to create tempdir")?;
        let root = temp_dir.path();
        for file in &FILES_TO_CREATE {
            let full_path = root.join(file);
            let parent = full_path.parent().unwrap();
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed to create test dir '{parent:?}'"))?;
            // use file path as file contents
            std::fs::write(&full_path, full_path.clone().into_os_string().as_bytes())
                .with_context(|| format!("failed to create test file '{full_path:?}'"))?;
        }

        // create demo config with home dir as target
        let conf = PackageConfig::new(root, BASE_DIRS.home_dir());
        conf.save_to_package()
            .context("failed to save test config")?;

        Ok(temp_dir)
    }

    #[test]
    fn test_try_from_package() -> anyhow::Result<()> {
        let package = make_tmp_tree().context("failed to make test package")?;
        let package_path = package.path();
        let conf = PackageConfig::try_from_package(package_path)
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
        assert!(!conf.no_create_dirs);
        assert_eq!(conf.link_type, LinkType::SymlinkAbsolute);

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
}
