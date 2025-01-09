use std::{path::PathBuf, str::FromStr};

use clap::Parser;
use directories_next::UserDirs;

/// Parses `&str` into a `PathBuf`. If the path begins with a `~`, it is expanded into the users
/// home directory.
///
/// # Arguments
///
/// - `s` - String slice to parse.
///
/// # Errors
///
/// No errors should occur. It returns a `Result` because that is the required signature for a `clap`
/// value parser, which this is.
///
/// # Panics
///
/// This function will panic if the users home directory cannot be found OR if `PathBuf::from_str`
/// fails, although neither should ever happen.
fn parse_and_expand_pathbuf(s: &str) -> Result<PathBuf, String> {
    let expanded = if let Some(rest) = s.strip_prefix('~') {
        let mut p = UserDirs::new()
            .unwrap_or_else(|| unreachable!("Failed to locate user home directory"))
            .home_dir()
            .to_path_buf();
        p.push(rest);
        p
    } else {
        PathBuf::from_str(s).unwrap_or_else(|_| unreachable!("Failed to convert &str to PathBuf"))
    };

    Ok(expanded)
}

#[derive(Debug, Parser)]
struct StowArgs {
    /// Package to stow. Can be a single file or a directory.
    #[arg(value_parser = parse_and_expand_pathbuf)]
    pub package: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let cli_args = StowArgs::parse();

    #[cfg(debug_assertions)]
    println!("{cli_args:#?}");

    Ok(())
}
