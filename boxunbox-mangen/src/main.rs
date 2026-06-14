use std::fs;

use boxunbox::{
    cli::{BoxUpCli, UnboxCli},
    utils::get_cargo_target,
};
use clap::CommandFactory;

fn main() -> anyhow::Result<()> {
    let target = get_cargo_target()?;
    let out_dir = target.join("man");
    fs::create_dir_all(&out_dir)?;

    let cmds = vec![UnboxCli::command(), BoxUpCli::command()];

    for cmd in cmds {
        let cmd_name = cmd
            .get_bin_name()
            .unwrap_or_else(|| cmd.get_name())
            .to_string();
        let man = clap_mangen::Man::new(cmd);
        let mut buffer: Vec<u8> = Vec::default();
        man.render(&mut buffer)?;

        let man_name = format!("{cmd_name}.1");
        let man_path = out_dir.join(man_name);
        fs::write(&man_path, buffer)?;
        println!("saved manpage to {}", man_path.display());
    }
    Ok(())
}
