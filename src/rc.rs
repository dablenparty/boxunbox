use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::Context;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct BoxUnboxRcArgs {
    pub target: PathBuf,
    pub include_dirs: bool,
    #[serde(with = "serde_regex")]
    pub ignore_pats: Vec<Regex>,
}

impl BoxUnboxRcArgs {
    pub fn parse_rc_file<P: AsRef<Path>>(rc_path: P) -> anyhow::Result<Self> {
        let rc_path = rc_path.as_ref();
        anyhow::ensure!(rc_path.exists(), "{rc_path:?} doesn't exist!");

        let rc_str = fs::read_to_string(rc_path)?;
        ron::from_str(&rc_str).with_context(|| format!("failed to parse rc file: {rc_path:?}"))
    }
}
