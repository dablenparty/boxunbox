use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;

/// Parses a `&str` slice as a PathBuf, expand `~` and environment variables, and clean the path.
fn cli_parse_pathbuf(s: &str) -> Result<PathBuf, String> {
    fn expand_pathbuf<S: AsRef<str>>(s: S) -> anyhow::Result<PathBuf> {
        let s = s.as_ref();
        let expanded = shellexpand::full(s).with_context(|| format!("failed to expand {s:?}"))?;
        let cleaned = path_clean::clean(expanded.as_ref());
        Ok(cleaned)
    }

    expand_pathbuf(s).map_err(|err| err.to_string())
}

#[derive(Debug, Parser)]
pub struct BoxUnboxCli {
    #[arg(required = true, value_parser = cli_parse_pathbuf)]
    pub package: PathBuf,
    #[arg(short, long, value_parser = cli_parse_pathbuf)]
    pub target: Option<PathBuf>,
}
