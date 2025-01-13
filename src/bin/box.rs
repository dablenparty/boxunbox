use std::{fs::DirEntry, path::Path};

use boxunbox::{cli::BoxUnboxArgs, get_package_entries, parse_rc_file, PackageEntry};
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
        package, target, ..
    } = &cli_args;

    let rc_path = package.join(".unboxrc");
    if rc_path.exists() {
        let rc_args = parse_rc_file(rc_path)?;

        #[cfg(debug_assertions)]
        println!("parsed rc file with args: {rc_args:#?}");

        // TODO: combine RC file and command line arguments
    }

    for res in get_package_entries(&cli_args)? {
        match res {
            Ok(pkg_entry) => {
                let PackageEntry { fs_entry } = pkg_entry;
                if let Err(error) = box_package_entry(&fs_entry, target) {
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
