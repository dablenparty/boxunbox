use std::{fs::DirEntry, path::Path};

use anyhow::Context;
use boxunbox::{cli::BoxUnboxArgs, get_package_entries, PackageEntry};
use clap::Parser;

/// Unboxes a package entry in `target`. The `pkg_entry`'s file name is used for the name of the symlink.
///
/// # Arguments
///
/// - `pkg_entry` - The [`DirEntry`] to unbox.
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
    let cli_args = BoxUnboxArgs::parse();

    #[cfg(debug_assertions)]
    println!("{cli_args:#?}");

    let target = cli_args.target.as_path();

    for res in get_package_entries(&cli_args)? {
        match res {
            Ok(pkg_entry) => {
                let PackageEntry { fs_entry } = pkg_entry;
                if let Err(error) = unbox_package_entry(&fs_entry, target) {
                    eprintln!(
                        "error unboxing {}: {error:?}",
                        fs_entry.file_name().to_string_lossy()
                    );
                }
            }
            Err(error) => eprintln!("error reading entry: {error:?}"),
        }
    }

    Ok(())
}
