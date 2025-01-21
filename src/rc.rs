use std::{
    fs,
    path::{Path, PathBuf},
    sync::LazyLock,
};

use anyhow::Context;
use regex::Regex;
use ron::ser::PrettyConfig;
use serde::{de::Error, Deserialize, Deserializer, Serialize};

use crate::{expand_and_clean_pathbuf, package::PackageOptions};

fn deserialize_target<'de, D>(d: D) -> Result<PathBuf, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(d)?;

    expand_and_clean_pathbuf(s).map_err(D::Error::custom)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BoxUnboxRcArgs {
    #[serde(deserialize_with = "deserialize_target")]
    pub target: PathBuf,
    pub include_dirs: bool,
    #[serde(with = "serde_regex")]
    pub ignore_pats: Vec<Regex>,
}

impl From<PackageOptions> for BoxUnboxRcArgs {
    fn from(value: PackageOptions) -> Self {
        Self {
            target: value.target,
            include_dirs: value.include_dirs,
            ignore_pats: value.ignore_pats,
        }
    }
}

impl BoxUnboxRcArgs {
    /// Save these arguments to an RC file.
    pub fn save_rc_file<P: AsRef<Path>>(&self, rc_path: P) -> anyhow::Result<()> {
        static PRETTY_CONFIG: LazyLock<PrettyConfig> = LazyLock::new(|| {
            PrettyConfig::new()
                .separate_tuple_members(true)
                .struct_names(true)
        });
        let rc_path = rc_path.as_ref();

        let rc_str = ron::ser::to_string_pretty(self, PRETTY_CONFIG.clone())
            .with_context(|| format!("failed to convert to RON format: {self:?}"))?;
        fs::write(rc_path, rc_str)
            .with_context(|| format!("failed to write to rc file {rc_path:?}"))?;

        Ok(())
    }

    /// Parse a `.unboxrc` file and return the parsed arguments.
    ///
    /// The file is expected to be in [RON format](https://github.com/ron-rs/ron).
    ///
    /// # Arguments
    ///
    /// - `rc_path` - Path to the RC file.
    ///
    /// # Errors
    ///
    /// An error is returned if:
    ///
    /// - `rc_path` doesn't exist
    /// - `rc_path` cannot be read (see [`std::fs::read_to_string`])
    /// - `rc_path` cannot be parsed as a valid RON file.
    pub fn parse_rc_file<P: AsRef<Path>>(rc_path: P) -> anyhow::Result<Self> {
        let rc_path = rc_path.as_ref();
        anyhow::ensure!(rc_path.exists(), "{rc_path:?} doesn't exist!");

        let rc_str = fs::read_to_string(rc_path)?;
        // the pathbuf parser uses the cwd to canonicalize; so, since the rc file is meant
        // to be relative to the package, we set the cwd to the package dir before parsing
        // the RC file.
        let args = {
            let old_cwd = std::env::current_dir().context("failed to get cwd")?;

            let pkg_dir = rc_path
                .parent()
                .with_context(|| format!("path {rc_path:?} has no parent"))?;
            std::env::set_current_dir(pkg_dir)
                .with_context(|| format!("failed to set cwd to {pkg_dir:?}"))?;

            let args = ron::from_str(&rc_str)
                .with_context(|| format!("failed to parse rc file: {rc_path:?}"))?;

            std::env::set_current_dir(&old_cwd)
                .with_context(|| format!("failed to reset cwd to {old_cwd:?}"))?;

            args
        };

        Ok(args)
    }
}
