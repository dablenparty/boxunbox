use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;
use cli::BoxUnboxCli;
use package::PackageConfig;
use rc::{errors::ParseError, BoxUnboxRc};

mod cli;
mod package;
mod rc;

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

    let rc = match BoxUnboxRc::try_parse_from_package(&package) {
        Ok(rc) => {
            #[cfg(debug_assertions)]
            println!("rc={rc:#?}");
            rc
        }
        Err(err) => {
            if let ParseError::RcFileNotFound(rc_path) = err {
                eprintln!(
                    "didn't find RC file @ {}, creating default...",
                    rc_path.display()
                );
                let default_rc = BoxUnboxRc::default();
                // if it doesn't exist, perform the merge, then re-extract the RC args
                // and write them to the file.
                let writeable_rc = BoxUnboxRc::from(PackageConfig::from_parts(&cli, &default_rc));
                writeable_rc.save_package_rc(&package)?;

                Ok(default_rc)
            } else {
                Err(err)
            }?
        }
    };

    let pkg_config = PackageConfig::from_parts(&cli, &rc);

    #[cfg(debug_assertions)]
    println!("pkg_config={pkg_config:#?}");

    // TODO: merge RC file and CLI args into one settings struct
    // TODO: unbox package
    // TODO: better documentation, organization, and error handling
    // TODO: regex ignore patterns

    Ok(())
}
