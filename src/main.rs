#![warn(clippy::all, clippy::pedantic)]

use std::path::Path;

use clap::Parser;
use cli::{BoxUnboxCli, ColorOverride};

mod cli;
mod constants;
mod package;
mod utils;

/// Unbox the package.
///
/// # Arguments
///
/// - `package` - Package directory to unbox.
fn unbox(package: &Path) {
    todo!()
}

fn main() {
    let cli = BoxUnboxCli::parse();

    #[cfg(debug_assertions)]
    println!("cli={cli:#?}");

    let BoxUnboxCli {
        packages,
        color_override,
        ..
    } = cli;

    match color_override {
        ColorOverride::Always => colored::control::set_override(true),
        ColorOverride::Auto => colored::control::unset_override(),
        ColorOverride::Never => colored::control::set_override(false),
    }

    for package in &packages {
        unbox(package);
    }
}
