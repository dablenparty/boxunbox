use std::path::PathBuf;

use clap::Parser;

use crate::expand_into_pathbuf;

/**
Parses a `&str` slice as a [`PathBuf`], expand `~` and environment variables and clean the path.

# Arguments

- `s` - `&str` slice.
*/
fn cli_parse_pathbuf(s: &str) -> Result<PathBuf, String> {
    expand_into_pathbuf(s).map_err(|err| err.to_string())
}

#[derive(Debug, Parser)]
pub struct BoxUnboxCli {
    #[arg(required = true, value_parser = cli_parse_pathbuf)]
    pub package: PathBuf,
    #[arg(short, long, value_parser = cli_parse_pathbuf)]
    pub target: Option<PathBuf>,
}
