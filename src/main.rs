use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;
use cli::BoxUnboxCli;

mod cli;

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

fn main() {
    let cli = BoxUnboxCli::parse();

    #[cfg(debug_assertions)]
    println!("cli={cli:#?}");
}
