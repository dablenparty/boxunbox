use std::{
    fs::DirEntry,
    path::{Path, PathBuf},
};

use boxunbox::cli::parse_and_expand_pathbuf;
use clap::{Parser, ValueHint};

/// `box`es a directory back up after being `unbox`ed. Iterates over the files in `package` and removes all matching symlinks from `target`.
#[derive(Debug, Parser, Clone)]
struct BoxArgs {
    /// Package to box. Can be a single file or a directory.
    #[arg(value_parser = parse_and_expand_pathbuf, value_hint = ValueHint::AnyPath)]
    pub package: PathBuf,
    /// Target to box up. Must be a directory.
    #[arg(default_value = "~", value_parser = parse_and_expand_pathbuf, value_hint = ValueHint::DirPath)]
    pub target: PathBuf,
    /// Include directories when unboxing.
    #[arg(short = 'd', long, default_value_t = false)]
    pub include_dirs: bool,
}

/// Boxes a package entry up from `target`. The `pkg_entry`'s file name is used to make the symlink
/// path.
///
/// # Arguments
///
/// - `pkg_entry` - The [`DirEntry`] to box.
/// - `target` - The `&Path` to box things up from.
///
/// # Errors
///
/// An error is returned if one occurs removing the symlink.
#[inline]
fn box_package_entry(pkg_entry: &DirEntry, target: &Path) -> anyhow::Result<()> {
    let link_path = target.join(pkg_entry.file_name());

    std::fs::remove_file(link_path)?;

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let cli_args = BoxArgs::parse();

    #[cfg(debug_assertions)]
    println!("cli_args={cli_args:#?}");

    let BoxArgs {
        package,
        target,
        include_dirs,
    } = cli_args;

    anyhow::ensure!(package.is_dir(), "{package:?} is not a directory");

    for res in package.read_dir()? {
        match res {
            Ok(entry) => {
                if !include_dirs && entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                    return Ok(());
                }
                if let Err(error) = box_package_entry(&entry, &target) {
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
