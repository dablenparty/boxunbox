use std::{
    fs::{DirEntry, File},
    io::{BufRead, BufReader},
    iter,
    path::Path,
    sync::LazyLock,
};

use anyhow::Context;
use clap::Parser;
use cli::{BoxUnboxArgs, BoxUnboxRcArgs};
use regex::Regex;

pub mod cli;

#[derive(Debug)]
pub struct PackageEntry {
    pub fs_entry: DirEntry,
}

impl From<DirEntry> for PackageEntry {
    fn from(value: DirEntry) -> Self {
        Self { fs_entry: value }
    }
}

/// Gets all entries to either `box` or `unbox` from a package.
///
/// # Arguments
///
/// - `args` - Arguments for filtering the packages.
///
/// # Errors
///
/// An error is returned if one occurs when reading the package directory. Errors for individual
/// [`DirEntry`]'s do not end this function and are instead collected into the returned `Vec`.
pub fn get_package_entries(
    args: &BoxUnboxArgs,
) -> anyhow::Result<Vec<anyhow::Result<PackageEntry>>> {
    static RC_REGEX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new("^\\.unboxrc$").expect(".unboxrc regex failed to compile"));

    let BoxUnboxArgs {
        package,
        include_dirs,
        ignore_pats,
        ..
    } = args;

    anyhow::ensure!(package.is_dir(), "{package:?} is not a directory");

    let pkg_entries = package
        .read_dir()
        .with_context(|| format!("Failed to read directory {package:?}"))?
        .filter_map(|res| {
            if let Ok(ref ent) = res {
                // need utf8 string for regex
                let file_name = ent.file_name();
                // shadow previous name to get around temp value error without keeping it
                let file_name = file_name.to_string_lossy();

                if ignore_pats
                    .iter()
                    .chain(iter::once(&RC_REGEX.clone()))
                    .any(|re| re.is_match(&file_name))
                    || (!include_dirs && ent.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
                {
                    return None;
                }
            }
            Some(
                res.map(PackageEntry::from)
                    .context("Failed to read dir entry"),
            )
        })
        .collect();

    Ok(pkg_entries)
}

/// Parse and tokenize a `.unboxrc` file. Arguments are split by whitespace while respecting
/// quotes, albeit simply.
///
/// # Arguments
///
/// - `file_path` - [`Path`] to the `.unboxrc` file.
///
/// # Errors
///
/// An error is returned in any of the following situations:
///
/// - `file_path` cannot be opened (see [`std::fs::OpenOptions::open`])
fn tokenize_rc_file<P: AsRef<Path>>(file_path: P) -> anyhow::Result<Vec<String>> {
    // Option holding the last quote character found
    let mut last_quote = None;
    // open in read-only mode
    let mut combined_args: Vec<String> = vec![];
    let mut current_token = String::new();

    let fp = File::open(file_path)?;
    let reader = BufReader::new(fp);

    // split on lines for ease of use
    for (line_num, line) in reader.lines().enumerate() {
        let line = line?;
        let mut chars = line.chars().peekable();

        while let Some(c) = chars.next() {
            /*
            The entire tokenizer is basically this match statement. It works as follows:
            `c` is appended to `current_token`. If `c` is whitespace, then `current_token` is
            saved and cleared and `c` is skipped. If `c` is a quote, then everything after
            (including whitespace) is assumed to be part of the same token. If no matching
            closing quote is found, an error is raised. Spaces and other characters can be
            escaped with a backslash `\`.
            */
            match c {
                '"' | '\'' | '`' => {
                    if let Some(q) = last_quote {
                        // remove the last quote if we found its match (string is complete)
                        if q == c {
                            let _ = last_quote.take();
                        } else {
                            current_token.push(c);
                        }
                    } else {
                        last_quote = Some(c);
                    }
                }
                '\\' if last_quote.is_some() => {
                    if let Some(next_char) = chars.peek() {
                        current_token.push(*next_char);
                        chars.next(); // Consume the peeked character
                    } else {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("Unterminated escape sequence at line {}", line_num),
                        )
                        .into());
                    }
                }
                _ if last_quote.is_none()
                    && c.is_whitespace()
                    && !current_token.trim().is_empty() =>
                {
                    // this token is done, save it and clear the buffer for the next one
                    combined_args.push(current_token.clone());
                    current_token.clear();
                }
                _ => current_token.push(c),
            }
        }

        if last_quote.is_none() && !current_token.trim().is_empty() {
            combined_args.push(current_token.clone());
            current_token.clear();
        }
    }

    if let Some(q) = last_quote {
        // TODO: better error here
        anyhow::bail!("Bad .unboxrc file format: found unbalanced quote {q:?}");
    }

    Ok(combined_args)
}

/// Parse a `.unboxrc` file and return the arguments. Arguments can either be on one line, separate
/// lines, or a combination of the two.
///
/// # Arguments
///
/// - `file_path` - A [`Path`] ref to the RC file.
///
/// # Errors
///
/// An error is returned for any of the following conditions:
///
/// - `file_path` cannot be read (see [`std::fs::read_to_string`]).
/// - `file_path` is not valid Unicode.
/// - The current working directory cannot be determined or changed.
///     - The CWD needs to be changed so that relative paths get canonicalized properly
pub fn parse_rc_file<P: AsRef<Path>>(file_path: P) -> anyhow::Result<BoxUnboxRcArgs> {
    let file_path = file_path.as_ref();

    let combined_args = tokenize_rc_file(file_path)?;

    /*
    I use a custom PathBuf parser that expands `~` and canonicalizes the path; however, that
    assumes that the path is being canonicalized from the dierctory the program was called
    from (i.e. the `cwd`). RC files are in the _package_ dirs, so the `cwd` is set to the
    package dir while parsing the RC file, then reset when done.
    */
    let old_cwd = std::env::current_dir()?;
    std::env::set_current_dir(file_path.parent().unwrap())?;

    // prepend the package name since clap requires a prog name to parse args properly.
    // TODO: failure prints usage string, stop that since these aren't actually command line args
    let parsed_args = BoxUnboxRcArgs::try_parse_from(
        iter::once(env!("CARGO_PKG_NAME").to_string()).chain(combined_args),
    )
    .with_context(|| format!("Failed to parse args from rc file: {file_path:?}"))?;

    std::env::set_current_dir(old_cwd)?;

    Ok(parsed_args)
}
