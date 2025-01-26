use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;
use cli::BoxUnboxCli;
use package::{errors::ParseError, PackageConfig};

mod cli;
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
            println!("rc={rc:#?}");
            rc
        }
        Err(err) => {
            if let ParseError::FileNotFound(rc_path) = err {
                eprintln!(
                    "didn't find RC file @ {}, creating default...",
                    rc_path.display()
                );

                todo!("create rc file since it doesn't exist");
            } else {
                Err(err)
            }?
        }
    };

    #[cfg(debug_assertions)]
    println!("pkg_config={pkg_config:#?}");

    // TODO: merge RC file and CLI args into one settings struct
    // TODO: unbox package
    // TODO: better documentation, organization, and error handling
    // TODO: regex ignore patterns

    Ok(())
}
