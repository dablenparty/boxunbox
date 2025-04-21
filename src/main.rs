#![warn(clippy::all, clippy::pedantic)]

use anyhow::Context;
use clap::Parser;
use cli::{BoxUnboxCli, ColorOverride};
use colored::Colorize;
use package::{PackageConfig, errors::ParseError, plan::UnboxPlan};

mod cli;
mod constants;
mod package;
mod utils;

fn main() -> anyhow::Result<()> {
    let cli = BoxUnboxCli::parse();

    #[cfg(debug_assertions)]
    println!("cli={cli:#?}");

    let BoxUnboxCli {
        color_override,
        dry_run,
        save_config,
        save_os_config,
        ..
    } = cli;

    match color_override {
        ColorOverride::Always => colored::control::set_override(true),
        ColorOverride::Auto => colored::control::unset_override(),
        ColorOverride::Never => colored::control::set_override(false),
    }

    let packages = cli
        .packages
        .iter()
        .map(|pkg| {
            if !pkg.is_dir() {
                anyhow::bail!("'{}' is not a directory", pkg.display());
            }

            pkg.canonicalize()
                .with_context(|| format!("failed to canonicalize {pkg:?}"))
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    for package in &packages {
        let pkg_config = match PackageConfig::try_from_package(package) {
            Ok(rc) => {
                #[cfg(debug_assertions)]
                println!("parsed_rc={rc:#?}");
                // TODO: better errors for this function

                rc.merge_with_cli(&cli)?
            }

            #[allow(unused_variables)]
            Err(ParseError::ConfigNotFound(rc_path)) => {
                let config = PackageConfig::new(package).merge_with_cli(&cli)?;

                #[cfg(debug_assertions)]
                {
                    eprintln!(
                        "didn't find RC file @ {}, creating default...",
                        rc_path.display()
                    );
                    eprintln!("config={config:#?}");
                }

                config
            }

            Err(err) => return Err(err.into()),
        };

        if save_config || save_os_config {
            let pkg_str = format!(
                "{}/",
                package
                    .file_name()
                    .map(|s| s.to_string_lossy())
                    .expect("package path has no basename")
            );
            println!("Saving config for {}", pkg_str.bright_green());
            pkg_config.save_to_package(package, save_os_config)?;
        }

        #[cfg(debug_assertions)]
        println!("pkg_config={pkg_config:#?}");

        let unbox_plan = UnboxPlan::new(pkg_config).context("failed to plan unboxing")?;

        // lay out the plan!
        println!("{unbox_plan}");
        if dry_run {
            println!("dry run, not executing");
            continue;
        }

        unbox_plan.execute().context("failed to execute plan")?;
    }

    Ok(())
}
