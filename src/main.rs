use clap::Parser;
use cli::BoxUnboxCli;

mod cli;

fn main() {
    let cli = BoxUnboxCli::parse();

    #[cfg(debug_assertions)]
    println!("cli={cli:#?}");
}
