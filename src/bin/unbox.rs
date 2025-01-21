use std::{fs::DirEntry, path::Path};

use boxunbox::{cli::BoxUnboxCli, get_package_entries, rc::BoxUnboxRcArgs, PackageEntry};
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
    let cli_args = BoxUnboxCli::parse();

    #[cfg(debug_assertions)]
    println!("{cli_args:#?}");

    let rc_path = cli_args.package.join(".unboxrc");
    if rc_path.exists() {
        let rc_args = BoxUnboxRcArgs::parse_rc_file(rc_path)?;

        #[cfg(debug_assertions)]
        println!("parsed rc file with args: {rc_args:#?}");

        // TODO: merge rc with main args
    }

    let BoxUnboxCli {
        dry_run,
        ref target,
        ..
    } = cli_args;

    for res in get_package_entries(&cli_args)? {
        match res {
            Ok(pkg_entry) if dry_run => {
                println!(
                    "unboxing {:?} -> {target:?}",
                    pkg_entry.fs_entry.path().display()
                );
            }
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
