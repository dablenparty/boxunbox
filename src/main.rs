#![warn(clippy::all, clippy::pedantic)]

use clap::Parser;
use cli::{BoxUnboxCli, ColorOverride};

mod cli;
mod constants;
mod package;
mod utils;

fn main() {
    let cli = BoxUnboxCli::parse();

    #[cfg(debug_assertions)]
    println!("cli={cli:#?}");

    let BoxUnboxCli { color_override, .. } = cli;

    match color_override {
        ColorOverride::Always => colored::control::set_override(true),
        ColorOverride::Auto => colored::control::unset_override(),
        ColorOverride::Never => colored::control::set_override(false),
    }
}
