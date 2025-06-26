#![warn(clippy::all, clippy::pedantic)]

use std::path::Path;

use anyhow::Context;
use boxunbox::cli::{BoxUnboxCli, ColorOverride};
use boxunbox::error::UnboxError;
use boxunbox::package::{self, PackageConfig};
use boxunbox::plan::UnboxPlan;
use boxunbox::utils::replace_home_with_tilde;
use clap::Parser;

/// Unbox the package.
///
/// # Arguments
///
/// - `package` - Package directory to unbox.
fn unbox(package: &Path, cli: &BoxUnboxCli) -> Result<(), UnboxError> {
    let config = match PackageConfig::init(package, cli) {
        Ok(config) => config,
        Err(package::error::ConfigRead::FileNotFound(_)) => {
            let mut conf = PackageConfig::new(package);
            conf.merge_with_cli(cli);
            conf
        }
        Err(err) => return Err(err.into()),
    };

    #[cfg(debug_assertions)]
    println!("{config:#?}");

    // OS config always takes precedence
    if cli.save_os_config {
        config.save_to_os_package()?;
    }

    if cli.save_config {
        config.save_to_package()?;
    }

    let unboxing_plan = UnboxPlan::plan_unboxing(config.clone(), cli)?;

    #[cfg(debug_assertions)]
    println!("{unboxing_plan:#?}");

    println!("{}", unboxing_plan.display(&config));

    if cli.dry_run {
        return Err(UnboxError::DryRun);
    }

    unboxing_plan.unbox()?;

    Ok(())
}

fn main() -> anyhow::Result<()> {
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
        let canon_package = dunce::canonicalize(package)?;
        unbox(&canon_package, &cli).with_context(|| {
            format!(
                "failed to unbox {}",
                replace_home_with_tilde(&canon_package)
            )
        })?;
    }

    Ok(())
}
