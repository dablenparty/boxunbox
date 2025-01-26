use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;
use cli::BoxUnboxCli;
use rc::BoxUnboxRc;

mod cli;
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
            println!("rc={rc:#?}");
            rc
        }
        Err(err) => {
            if let rc::RcParseError::RcFileNotFound(rc_path) = err {
                eprintln!(
                    "didn't find RC file @ {}, creating default...",
                    rc_path.display()
                );
                let default_rc = BoxUnboxRc::default();
                default_rc.save_package_rc(&package)?;
                Ok(default_rc)
            } else {
                Err(err)
            }?
        }
    };

    println!("rc={rc:#?}");

    // TODO: merge RC file and CLI args into one settings struct
    // TODO: unbox package
    // TODO: better documentation, organization, and error handling

    Ok(())
}
