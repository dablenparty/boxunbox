use std::{env, fs, io, path::PathBuf};

use boxunbox::cli::BoxUnboxCli;
use clap::CommandFactory;

fn main() -> io::Result<()> {
    // FIXME: creates target/man within this crate, but OUT_DIR doesn't work
    let out_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").ok_or(io::ErrorKind::NotFound)?)
        .join("target")
        .join("man");

    let cmd = BoxUnboxCli::command();

    let man = clap_mangen::Man::new(cmd);
    let mut buffer: Vec<u8> = Vec::default();
    man.render(&mut buffer)?;

    let man_name = format!("{}.1", env!("CARGO_PKG_NAME"));
    fs::create_dir_all(&out_dir)?;
    fs::write(out_dir.join(man_name), buffer)?;

    Ok(())
}
