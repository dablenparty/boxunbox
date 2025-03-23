#![warn(clippy::all, clippy::pedantic)]

use anyhow::Context;
use clap::Parser;
use cli::BoxUnboxCli;
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
        color: color_override,
        ref package,
        save_config,
        save_os_config,
        perform_box,
        ..
    } = cli;

    match color_override {
        cli::ColorOverride::Always => colored::control::set_override(true),
        cli::ColorOverride::Auto => colored::control::unset_override(),
        cli::ColorOverride::Never => colored::control::set_override(false),
    }

    let package = package
        .clone()
        .canonicalize()
        .with_context(|| format!("failed to canonicalize {package:?}"))?;

    let pkg_config = match PackageConfig::try_from_package(&package) {
        Ok(rc) => {
            #[cfg(debug_assertions)]
            println!("parsed_rc={rc:#?}");
            // TODO: better errors for this function

            rc.merge_with_cli(&cli)?
        }

        #[allow(unused_variables)]
        Err(ParseError::ConfigNotFound(rc_path)) => {
            let config = PackageConfig::try_from(cli.clone())?;

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
        println!("saving config...");
        pkg_config.save_to_package(&package, save_os_config)?;
    }

    #[cfg(debug_assertions)]
    println!("pkg_config={pkg_config:#?}");

    let unbox_plan = UnboxPlan::new(pkg_config).context("failed to plan unboxing")?;

    if perform_box {
        unbox_plan.box_up().context("failed to box up package")
    } else {
        unbox_plan.check_plan()?;
        // FIXME: https://github.com/dablenparty/boxunbox/issues/4
        // "Boxing" greedily removes all planned target links and doesn't care if they were created
        // by this program or if they already existed. This is destructive and not suitable for
        // rollback functionality.
        unbox_plan.execute().context("failed to execute unbox plan")
    }
}
