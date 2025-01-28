#![warn(clippy::all, clippy::pedantic)]

use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;
use cli::BoxUnboxCli;
use constants::BASE_DIRS;
use package::{errors::ParseError, PackageConfig};

mod cli;
mod constants;
mod package;

/**
Given a reference to a `&str` slice, expand `~` and environment variables, clean path
components, and return as a [`PathBuf`].

# Arguments

- `s` - `&str` slice.

# Errors

An error is returned if an environment variable cannot be found.
*/
fn expand_into_pathbuf<S: AsRef<str>>(s: S) -> anyhow::Result<PathBuf> {
    let s = s.as_ref();
    let expanded = shellexpand::full(s).with_context(|| format!("failed to expand {s:?}"))?;
    let cleaned = path_clean::clean(expanded.as_ref());
    Ok(cleaned)
}

fn main() -> anyhow::Result<()> {
    let cli = BoxUnboxCli::parse();

    #[cfg(debug_assertions)]
    println!("cli={cli:#?}");

    let package = cli
        .package
        .clone()
        .canonicalize()
        .with_context(|| format!("failed to canonicalize {:?}", cli.package))?;

    let pkg_config = match PackageConfig::try_from_package(&package) {
        Ok(rc) => {
            #[cfg(debug_assertions)]
            println!("parsed_rc={rc:#?}");
            rc.merge_with_cli(cli)
        }
        Err(err) => {
            if let ParseError::FileNotFound(rc_path) = err {
                let mut default_config = PackageConfig::new(&package).merge_with_cli(cli);
                #[cfg(debug_assertions)]
                {
                    eprintln!(
                        "didn't find RC file @ {}, creating default...",
                        rc_path.display()
                    );
                    eprintln!("default_config={default_config:#?}");
                }

                default_config.save_to_package(&package)?;
                default_config.target = BASE_DIRS.home_dir().to_path_buf();

                Ok(default_config)
            } else {
                Err(err)
            }?
        }
    };

    #[cfg(debug_assertions)]
    println!("pkg_config={pkg_config:#?}");

    // TODO: unbox package
    // TODO: regex ignore patterns

    Ok(())
}
