#![warn(clippy::all, clippy::pedantic)]

use anyhow::Context;
use clap::Parser;
use cli::BoxUnboxCli;
use constants::BASE_DIRS;
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
            rc.merge_with_cli(cli)
        }
        Err(err) => {
            if let ParseError::FileNotFound(rc_path) = err {
                let mut default_config = PackageConfig::new(&package).merge_with_cli(cli);
                #[cfg(debug_assertions)]
                {
                    eprintln!(
                        "didn't find RC file @ {}, creating default...",
                        rc_path.display()
                    );
                    eprintln!("default_config={default_config:#?}");
                }

                default_config.save_to_package(&package)?;
                default_config.target = BASE_DIRS.home_dir().to_path_buf();

                Ok(default_config)
            } else {
                Err(err)
            }?
        }
    };

    #[cfg(debug_assertions)]
    println!("pkg_config={pkg_config:#?}");

    // TODO: unbox package
    // TODO: regex ignore patterns

    Ok(())
}
