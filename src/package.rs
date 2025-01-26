use std::path::PathBuf;

use crate::{cli::BoxUnboxCli, rc::BoxUnboxRc};

#[derive(Debug)]
pub struct PackageConfig {
    pub package: PathBuf,
    pub target: PathBuf,
}

impl PackageConfig {
    /// Create a [`PackageConfig`] from [`BoxUnboxCli`] and [`BoxUnboxRc`] configs.
    ///
    /// # Arguments
    ///
    /// - `cli` - CLI args
    /// - `rc` - RC args
    ///
    pub fn from_parts(cli: &BoxUnboxCli, rc: &BoxUnboxRc) -> Self {
        Self {
            package: cli.package.clone(),
            target: cli.target.as_ref().unwrap_or(&rc.target).clone(),
        }
    }
}
