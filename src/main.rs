#![warn(clippy::all, clippy::pedantic)]

use std::{path::Path, process};

use clap::Parser;
use cli::{BoxUnboxCli, ColorOverride};
use colored::Colorize;
use error::UnboxError;
use package::{OldPackageConfig, PackageConfig};
use plan::plan_unboxing;

mod cli;
mod constants;
mod error;
mod package;
mod plan;
mod utils;

mod test_utils;

/// Unbox the package.
///
/// # Arguments
///
/// - `package` - Package directory to unbox.
fn unbox(package: &Path, cli: &BoxUnboxCli) -> Result<(), UnboxError> {
    let config = match PackageConfig::init(package, cli) {
        Ok(config) => config,
        Err(package::error::ConfigRead::FileNotFound(path_buf)) => {
            // TODO: Remove this conversion eventually
            eprintln!(
                "{}: {} not found, checking for old config...",
                "warn".yellow(),
                path_buf.display()
            );
            match OldPackageConfig::try_from(package.to_path_buf()) {
                Ok(old_config) => {
                    let save_note = if cli.save_config || cli.save_os_config {
                        "A converted config will be saved."
                    } else {
                        "Please use --save-config to save the converted config."
                    };
                    eprintln!("{}: parsed old config! {save_note}", "warn".yellow());

                    PackageConfig::from_old_package(package, old_config)
                }
                Err(err) => {
                    eprintln!("{}: error reading old config: {err}", "warn".yellow());
                    PackageConfig::new(package)
                }
            }
        }
        Err(err) => return Err(err.into()),
    };

    #[cfg(debug_assertions)]
    println!("{config:#?}");

    let unboxing_plan = plan_unboxing(config, cli)?;

    // TODO: prettier output
    println!("Here's the plan: {unboxing_plan:#?}");

    if cli.dry_run {
        return Err(UnboxError::DryRun);
    }

    todo!()
}

fn main() {
    let cli = BoxUnboxCli::parse();

    #[cfg(debug_assertions)]
    println!("cli={cli:#?}");

    let BoxUnboxCli {
        ref packages,
        color_override,
        ..
    } = cli;

    match color_override {
        ColorOverride::Always => colored::control::set_override(true),
        ColorOverride::Auto => colored::control::unset_override(),
        ColorOverride::Never => colored::control::set_override(false),
    }

    for package in packages {
        if let Err(err) = unbox(package, &cli) {
            eprintln!(
                "{}: failed to unbox {}: {err}",
                "err".bright_red(),
                package.display()
            );
        }
    }
}
