use std::{
    collections::HashSet,
    fs::{self, OpenOptions},
    io::{BufRead, BufReader},
    path::PathBuf,
};

use anyhow::Context;
use boxunbox::{
    cli::{BoxUpCli, ColorOverride},
    error::UnboxError,
    utils::replace_home_with_tilde,
};
use clap::Parser;
use colored::Colorize;

// FIXME: TESTING

fn main() -> anyhow::Result<()> {
    let cli = BoxUpCli::parse();

    #[cfg(debug_assertions)]
    println!("cli={cli:#?}");

    // TODO: include/exclude patterns
    // TODO: dry run
    let BoxUpCli {
        fail_fast,
        packages,
        color_override,
        keep_last_file,
        ..
    } = cli;

    match color_override {
        ColorOverride::Always => colored::control::set_override(true),
        ColorOverride::Auto => colored::control::unset_override(),
        ColorOverride::Never => colored::control::set_override(false),
    }

    for package in packages {
        let canon_package = dunce::canonicalize(package)?;
        let last_unboxing_file = canon_package.join(".bub.last");
        let last_unboxed_paths = {
            let lufd = OpenOptions::new()
                .create(false)
                .read(true)
                .open(&last_unboxing_file)
                .map_err(|err| UnboxError::Io {
                    path: last_unboxing_file.clone(),
                    source: err,
                })?;
            let bufreader = BufReader::new(lufd);
            // read each line into a HashSet to deduplicate
            let paths_set = bufreader
                .lines()
                .map(|res| res.map(PathBuf::from))
                .collect::<Result<HashSet<_>, _>>()?;
            let mut paths_vec = Vec::from_iter(paths_set);
            paths_vec.sort();
            paths_vec
        };

        #[cfg(debug_assertions)]
        println!("unboxed paths: {last_unboxed_paths:#?}");

        for path in &last_unboxed_paths {
            if path.is_dir() {
                continue;
            } else {
                match fs::remove_file(path)
                    .with_context(|| format!("failed to remove unboxed file: {path:?}"))
                {
                    Ok(()) => {}
                    Err(err) if fail_fast => return Err(err),
                    Err(err) => eprintln!(
                        "{}: failed to remove {}: {err}",
                        "warn".yellow(),
                        path.display()
                    ),
                }
            }
            println!(
                "successfully removed {}",
                replace_home_with_tilde(path).red()
            );
        }

        println!(
            "successfully boxed up {}",
            replace_home_with_tilde(canon_package).red()
        );

        let luf_string = replace_home_with_tilde(&last_unboxing_file);

        if keep_last_file {
            println!("keeping {}", luf_string.cyan());
        } else {
            fs::remove_file(&last_unboxing_file)
                .with_context(|| format!("failed to remove {last_unboxing_file:?}"))?;
            println!("removed last file list {}", luf_string.red());
        }
    }

    Ok(())
}
