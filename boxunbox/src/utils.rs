use std::{
    io,
    path::{Path, PathBuf},
};

use anyhow::Context;

use crate::constants::BASE_DIRS;

/// If the [`Path`] reference begins with the users home directory, it is replaced with a `~`. This
/// is kinda the opposite of [`expand_into_pathbuf`] and meant for printing.
///
/// # Arguments
///
/// - `p` - Path reference
pub fn replace_home_with_tilde<P: AsRef<Path>>(p: P) -> String {
    let path = p.as_ref();
    let home = BASE_DIRS.home_dir();
    if let Ok(tail) = path.strip_prefix(home) {
        PathBuf::from("~").join(tail)
    } else {
        path.to_path_buf()
    }
    .to_string_lossy()
    .to_string()
}

/**
Given a reference to a `&str` slice, expand `~` and environment variables, clean path
components, and return as a [`PathBuf`].

# Arguments

- `s` - `&str` slice.

# Errors

An error is returned if an environment variable cannot be found.
*/
pub fn expand_into_pathbuf<S: AsRef<str>>(s: S) -> anyhow::Result<PathBuf> {
    let s = s.as_ref();
    let expanded = expandenv::expand(s).with_context(|| format!("failed to expand {s:?}"))?;
    let cleaned = path_clean::clean(expanded);
    Ok(cleaned)
}

/**
Create a new symbolic (soft) link using OS-specific functions.

This is really just a wrapper function.

# Arguments

- `original` - Original path.
- `link` - Link path.

# Errors

See the following for error descriptions:

- Unix: [`std::os::unix::fs::symlink`]
- Windows: TODO
*/
pub fn os_symlink<P, Q>(original: P, link: Q) -> io::Result<()>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    let original = original.as_ref();
    let link = link.as_ref();
    // [`std::fs::soft_link`] works fine, but is weird on Windows. The documentation recommends
    // using OS-specific libraries to make intent explicit.
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(original, link)
    }

    #[cfg(windows)]
    {
        todo!("PackageConfig::unbox for Windows")
    }

    #[cfg(not(any(windows, unix)))]
    {
        unimplemented!()
    }
}
