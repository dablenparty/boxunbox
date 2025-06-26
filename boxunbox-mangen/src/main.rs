use std::{env, fs};

use boxunbox::{cli::BoxUnboxCli, utils::get_cargo_target};
use clap::CommandFactory;

fn main() -> anyhow::Result<()> {
    let cmd = BoxUnboxCli::command();

    let man = clap_mangen::Man::new(cmd);
    let mut buffer: Vec<u8> = Vec::default();
    man.render(&mut buffer)?;

    let target = get_cargo_target()?;
    let out_dir = target.join("man");

    let man_name = format!("{}.1", env!("CARGO_PKG_NAME"));
    fs::create_dir_all(&out_dir)?;
    let man_path = out_dir.join(man_name);
    fs::write(&man_path, buffer)?;
    println!("saved manpage to {}", man_path.display());

    Ok(())
}
