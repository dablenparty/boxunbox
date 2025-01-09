use clap::Parser;

#[derive(Debug, Parser)]
struct StowArgs {}

fn main() -> anyhow::Result<()> {
    let cli_args = StowArgs::parse();

    #[cfg(debug_assertions)]
    println!("{cli_args:#?}");

    Ok(())
}
