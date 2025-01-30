#![warn(clippy::all, clippy::pedantic)]

use anyhow::Context;
use clap::Parser;
use cli::BoxUnboxCli;
use package::{errors::ParseError, PackageConfig};

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
        .with_context(|| format!("failed to canonicalize {:?}", cli.package))?;

    let pkg_config = match PackageConfig::try_from_package(&package) {
        Ok(rc) => {
            #[cfg(debug_assertions)]
            println!("parsed_rc={rc:#?}");
            // TODO: better errors for this function
            rc.merge_with_cli(cli)?
        }
        Err(err) => {
            if let ParseError::FileNotFound(rc_path) = err {
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

                Ok(config)
            } else {
                Err(err)
            }?
        }
    };

    #[cfg(debug_assertions)]
    println!("pkg_config={pkg_config:#?}");

    // TODO: regex ignore patterns
    pkg_config.unbox().context("failed to unbox")?;

    Ok(())
}
