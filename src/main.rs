#![warn(clippy::all, clippy::pedantic)]

use anyhow::Context;
use clap::Parser;
use cli::BoxUnboxCli;
use package::{errors::ParseError, plan::UnboxPlan, PackageConfig};

mod cli;
mod constants;
mod package;
mod utils;

fn main() -> anyhow::Result<()> {
    let cli = BoxUnboxCli::parse();

    #[cfg(debug_assertions)]
    println!("cli={cli:#?}");

    let package = cli
        .package
        .clone()
        .canonicalize()
        .with_context(|| format!("failed to canonicalize {:?}", &cli.package))?;
    let do_box = cli.perform_box;

    let pkg_config = match PackageConfig::try_from_package(&package) {
        Ok(rc) => {
            #[cfg(debug_assertions)]
            println!("parsed_rc={rc:#?}");
            // TODO: better errors for this function
            let config = rc.merge_with_cli(&cli)?;
            if cli.save_config {
                println!("saving config...");
                config.save_to_package(&package)?;
            }

            config
        }

        #[allow(unused_variables)]
        Err(ParseError::FileNotFound(rc_path)) => {
            let config = PackageConfig::try_from(cli)?;
            #[cfg(debug_assertions)]
            {
                eprintln!(
                    "didn't find RC file @ {}, creating default...",
                    rc_path.display()
                );
                eprintln!("config={config:#?}");
            }

            config.save_to_package(&package)?;

            config
        }

        Err(err) => return Err(err.into()),
    };

    #[cfg(debug_assertions)]
    println!("pkg_config={pkg_config:#?}");

    let unbox_plan = UnboxPlan::try_from_package(&package, pkg_config)
        .context("failed to create unboxing plan")?;

    if do_box {
        unbox_plan.rollback().context("failed to box up package")
    } else {
        unbox_plan.check_plan()?;
        unbox_plan
            .execute()
            .context("failed to execute unbox plan")
            .or_else(|err| {
                println!("An error occurred while executing the unboxing plan: {err:?}");
                unbox_plan
                    .rollback()
                    .context("failed to rollback unbox plan")
            })
    }
}
