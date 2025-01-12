use std::{
    fs::DirEntry,
    path::{Path, PathBuf},
};

use boxunbox::cli::parse_and_expand_pathbuf;
use clap::{Parser, ValueHint};

#[derive(Debug, Parser, Clone)]
struct UnboxArgs {
    /// Package to unbox. Can be a single file or a directory.
    #[arg(value_parser = parse_and_expand_pathbuf, value_hint = ValueHint::AnyPath)]
    pub package: PathBuf,
    /// Target directory to unbox the package to. Must be a directory.
    #[arg(default_value = "~", value_parser = parse_and_expand_pathbuf, value_hint = ValueHint::DirPath)]
    pub target: PathBuf,
    /// Include directories when unboxing.
    #[arg(short = 'd', long, default_value_t = false)]
    pub include_dirs: bool,
}

/// Unboxes a package entry in `target`. The `pkg_entry`'s file name is used for the name of the symlink.
///
/// # Arguments
///
/// - `pkg_entry` - The `DirEntry` to unbox.
/// - `target` - The `&Path` to unbox in. Must be a directory.
///
/// # Errors
///
/// An error is returned if there is an error creating the
/// symlink.
#[inline]
fn unbox_package_entry(pkg_entry: &DirEntry, target: &Path) -> anyhow::Result<()> {
    let path = pkg_entry.path();
    let link_path = target.join(pkg_entry.file_name());

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(path, link_path)?;
    }

    #[cfg(windows)]
    {
        todo!("implement windows?")
    }

    #[cfg(all(not(windows), not(unix)))]
    {
        std::fs::soft_link(path, link_path)?;
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let cli_args = UnboxArgs::parse();

    #[cfg(debug_assertions)]
    println!("{cli_args:#?}");

    let UnboxArgs {
        include_dirs,
        ref package,
        target,
    } = cli_args;

    anyhow::ensure!(package.is_dir(), "{package:?} is not a directory");

    for res in package.read_dir()? {
        match res {
            Ok(entry) => {
                if !include_dirs && entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                    return Ok(());
                }
                if let Err(error) = unbox_package_entry(&entry, &target) {
                    eprintln!(
                        "error unboxing {}: {error:?}",
                        entry.file_name().to_string_lossy()
                    );
                }
            }
            Err(error) => eprintln!("error reading entry: {error:?}"),
        }
    }

    Ok(())
}
