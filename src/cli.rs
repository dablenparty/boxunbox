use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
pub struct BoxUnboxCli {
    #[arg(required = true)]
    pub package: PathBuf,
    #[arg(short, long)]
    pub target: Option<PathBuf>,
}
