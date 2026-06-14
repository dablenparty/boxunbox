use std::{fs, io::BufWriter};

use boxunbox::{
    cli::{BoxUpCli, UnboxCli},
    utils::get_cargo_target,
};
use clap::{CommandFactory, ValueEnum};
use clap_complete::Shell;

fn main() -> anyhow::Result<()> {
    let cargo_target = get_cargo_target()?;
    let out_dir = cargo_target.join("completions");
    fs::create_dir_all(&out_dir)?;
    let cmds = vec![UnboxCli::command(), BoxUpCli::command()];
    for mut command in cmds {
        for shell in Shell::value_variants() {
            let name = command
                .get_bin_name()
                .unwrap_or_else(|| command.get_name())
                .to_string();

            let out_path = out_dir.join(&name).with_extension(shell.to_string());
            let fd = fs::File::create(&out_path)?;
            let mut writer = BufWriter::new(fd);
            println!("generating completions file {}", out_path.display());
            clap_complete::generate(*shell, &mut command, name, &mut writer);
        }
    }

    Ok(())
}
