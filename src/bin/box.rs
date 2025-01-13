use std::{fs::DirEntry, path::Path};

use anyhow::Context;
use boxunbox::cli::BoxUnboxArgs;
use clap::Parser;

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
    let cli_args = BoxUnboxArgs::parse();

    #[cfg(debug_assertions)]
    println!("cli_args={cli_args:#?}");

    let BoxUnboxArgs {
        package,
        target,
        include_dirs,
    } = cli_args;

    anyhow::ensure!(package.is_dir(), "{package:?} is not a directory");

    for res in package
        .read_dir()
        .with_context(|| format!("Failed to read directory: {package:?}"))?
    {
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
