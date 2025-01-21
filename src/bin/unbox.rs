use std::{fs::DirEntry, path::Path};

use anyhow::Context;
use boxunbox::{cli::BoxUnboxCli, package::PackageOptions, rc::BoxUnboxRcArgs, PackageEntry};
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

    anyhow::ensure!(link_path.exists(), "target {link_path:?} does not exist");

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&path, &link_path)
            .with_context(|| format!("failed unboxing {path:?} to {link_path:?}"))?;
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
    let do_dry_run = cli_args.dry_run;

    // if the RC file doesn't exist, make it from the command line options (--target is required)
    // if it does, try to parse it and merge it with the CLI args
    let package_opts = if !rc_path.exists() {
        anyhow::ensure!(cli_args.target.is_some(), "Failed to unbox {rc_path:?}: no `.unboxrc` file found. Note: the --target flag must be passed the first time a package is unboxed.");

        let opts = PackageOptions::try_from(cli_args).context("failed to create PackageOptions")?;
        let rc_args = BoxUnboxRcArgs::from(opts.clone());
        rc_args
            .save_rc_file(&rc_path)
            .with_context(|| format!("failed to save rc file {rc_path:?}"))?;

        println!("saved rc file: {}", rc_path.display());

        opts
    } else {
        let rc_args = BoxUnboxRcArgs::parse_rc_file(rc_path)?;

        #[cfg(debug_assertions)]
        println!("parsed rc file with args: {rc_args:#?}");

        PackageOptions::from_parts(cli_args, rc_args)
    };

    #[cfg(debug_assertions)]
    println!("parsed package options: {package_opts:#?}");

    let target = package_opts.target.as_path();

    for res in package_opts.get_package_entries()? {
        match res {
            Ok(pkg_entry) if do_dry_run => {
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
