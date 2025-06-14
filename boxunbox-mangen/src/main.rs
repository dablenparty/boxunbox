use std::{env, ffi::OsString, fs, path::Path, process};

use boxunbox::cli::BoxUnboxCli;
use clap::CommandFactory;

fn main() -> anyhow::Result<()> {
    #[cfg(debug_assertions)]
    const CARGO_PROFILE: &str = "debug";

    #[cfg(not(debug_assertions))]
    const CARGO_PROFILE: &str = "release";

    let cmd = BoxUnboxCli::command();

    let man = clap_mangen::Man::new(cmd);
    let mut buffer: Vec<u8> = Vec::default();
    man.render(&mut buffer)?;

    let cargo_output = process::Command::new(env!("CARGO"))
        .args(["locate-project", "--workspace", "--message-format=plain"])
        .output()?;

    if !cargo_output.status.success() {
        if let Some(code) = cargo_output.status.code() {
            anyhow::bail!("cargo locate-project exited with code {code:?}");
        } else {
            anyhow::bail!("cargo locate-project exited by signal");
        }
    }

    #[cfg(unix)]
    let workspace_manifest_path = <OsString as std::os::unix::ffi::OsStringExt>::from_vec(
        cargo_output.stdout.trim_ascii_end().to_vec(),
    );

    #[cfg(windows)]
    let workspace_manifest_path = todo!("create OsString from locate-project output on Windows");

    let out_dir = Path::new(&workspace_manifest_path)
        .parent()
        .expect("cargo locate-project output should be a file path")
        .join("target")
        .join(CARGO_PROFILE)
        .join("man");

    let man_name = format!("{}.1", env!("CARGO_PKG_NAME"));
    fs::create_dir_all(&out_dir)?;
    let man_path = out_dir.join(man_name);
    fs::write(&man_path, buffer)?;
    println!("saved manpage to {}", man_path.display());

    Ok(())
}
