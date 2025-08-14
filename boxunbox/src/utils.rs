use std::{
    ffi::OsString,
    io,
    path::{Path, PathBuf},
    process,
};

use anyhow::Context;

use crate::constants::BASE_DIRS;

/// Returns the cargo `target` directory as a [`PathBuf`] for the current profile (`debug` or
/// `release`). This works by calling `cargo locate-project` to get the workspace root.
///
/// # Errors
///
/// An error is returned if `cargo locate-project` returns a non-zero exit code.
///
/// # Panics
///
/// This function will panic if the directory returned by `cargo locate-project` is not an absolute
/// path to a directory.
pub fn get_cargo_target() -> anyhow::Result<PathBuf> {
    #[cfg(debug_assertions)]
    const CARGO_PROFILE: &str = "debug";

    #[cfg(not(debug_assertions))]
    const CARGO_PROFILE: &str = "release";

    let cargo_output = process::Command::new(env!("CARGO"))
        .args(["locate-project", "--workspace", "--message-format=plain"])
        .output()?;

    if !cargo_output.status.success() {
        if let Some(code) = cargo_output.status.code() {
            anyhow::bail!("cargo locate-project exited with code {code:?}");
        }
        anyhow::bail!("cargo locate-project exited by signal");
    }

    #[cfg(unix)]
    let workspace_manifest_path = <OsString as std::os::unix::ffi::OsStringExt>::from_vec(
        cargo_output.stdout.trim_ascii_end().to_vec(),
    );

    #[cfg(windows)]
    let workspace_manifest_path = <OsString as std::os::windows::ffi::OsStringExt>::from_wide(
        &cargo_output
            .stdout
            .trim_ascii_end()
            .iter()
            .copied()
            .map(u16::from)
            .collect::<Vec<_>>(),
    );

    Ok(Path::new(&workspace_manifest_path)
        .parent()
        .expect("cargo locate-project output should be a file path")
        .join("target")
        .join(CARGO_PROFILE))
}

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
- Windows: [`std::os::windows::fs::symlink_dir`] and [`std::os::windows::fs::symlink_file`]
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
        if original.is_dir() {
            std::os::windows::fs::symlink_dir(original, link)
        } else {
            std::os::windows::fs::symlink_file(original, link)
        }
    }

    #[cfg(not(any(windows, unix)))]
    {
        unimplemented!()
    }
}
