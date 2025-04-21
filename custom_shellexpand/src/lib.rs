#![warn(clippy::all, clippy::pedantic)]

use std::{
    collections::VecDeque,
    ffi::{OsStr, OsString},
    path::PathBuf,
    sync::LazyLock,
};

use directories_next::BaseDirs;
use regex::Regex;

use crate::errors::ExpandError;

pub mod errors;

/// Mimic's the behavior of [`PathBuf::components`] while respecting curly braces.
fn __parse_path_components_with_braces(s: &str) -> Vec<OsString> {
    let mut brace_depth: u8 = 0;
    let mut components = Vec::<OsString>::new();
    let mut comp = OsString::new();
    let mut comp_is_envvar = false;
    let mut char_iter = s.chars().peekable();

    // if path starts with path sep, consider it a path component
    // and remove it from the iterator
    if char_iter.peek().copied() == Some(std::path::MAIN_SEPARATOR) {
        components.push(std::path::MAIN_SEPARATOR.to_string().into());
        let _ = char_iter.next();
    }

    for c in char_iter {
        match c {
            std::path::MAIN_SEPARATOR if brace_depth == 0 => {
                components.push(comp.clone());
                comp.clear();
                comp_is_envvar = false;
                continue;
            }

            '$' => comp_is_envvar = true,
            '{' if comp_is_envvar => brace_depth = brace_depth.saturating_add(1),
            '}' if comp_is_envvar => brace_depth = brace_depth.saturating_sub(1),

            _ => {}
        }

        comp.push(c.to_string());
    }

    components.push(comp.clone());

    components
}

/// Convert a `&str` slice into a `PathBuf`, expanding envvars and the leading tilde `~`, if it
/// is there.
///
/// The tilde (`~`) expands into the users home directory as defined by [`directories_next::BaseDirs::home_dir`].
///
/// Environment variables expand into their value, optionally expanding a fallback value if the var
/// cannot be read. Envvars may contain letters, numbers, and underscores (`_`), but they _must_ start
/// with either a letter or an underscore after the dollar sign (`$`). Although more complicated
/// syntax is technically allowed by most programming languages, I will not be supporting anything
/// other than this basic structure because this is what most shells support and if you're doing
/// something different, ask yourself why.
///
/// # Arguments
///
/// - `s`: String to expand and convert
///
/// # Errors
///
/// An error is returned if:
///
/// - An envvar cannot be expanded
/// - You don't have a home directory
pub fn expand(s: &str) -> Result<PathBuf, ExpandError> {
    /// Lazy wrapper around [`directories_next::BaseDirs::new`].
    static BASE_DIRS: LazyLock<BaseDirs> =
        LazyLock::new(|| BaseDirs::new().expect("failed to locate users home directory"));
    static ENVVAR_REGEX: LazyLock<Regex> = LazyLock::new(|| {
        /*
         * TODO: put examples into doctests
         * Example matches:
         * 1. $ENV_VAR
         * 2. ${ENV_VAR}
         * 3. ${MISSING_VAR:-$ENV_VAR}
         * 4. ${MISSING_VAR:-~/path/to/file}
         *
         * There are three capture groups:
         * 1. The environment variable (minus the $)
         *    ([\w_][\w\d_]*)
         * 2. Ignore this group
         * 3. The fallback value
         *    (.*?)
         */
        Regex::new(r"\$\{?([a-zA-Z_]\w*)(:-(.*?))?\}?$").expect("invalid envvar regex")
    });

    let comp_strs = __parse_path_components_with_braces(s);
    let mut expanded_comps = VecDeque::with_capacity(comp_strs.len());

    for comp in comp_strs {
        let path = if let Some(captures) = ENVVAR_REGEX.captures(&comp.to_string_lossy()) {
            let envvar = captures
                .get(1)
                .and_then(|m| if m.is_empty() { None } else { Some(m.as_str()) })
                .ok_or(ExpandError::EmptyEnvvarCapture)?;

            #[cfg(debug_assertions)]
            println!("expanding envvar '{envvar:?}'");

            let envvar_value = if let Some(value) = std::env::var_os(envvar) {
                #[cfg(debug_assertions)]
                println!("{envvar:?}={value:?}");

                value
            } else if let Some(fallback) = captures.get(3) {
                let fallback = fallback.as_str();
                #[cfg(debug_assertions)]
                println!("failed to expand '{envvar:?}', found fallback '{fallback:?}'");

                expand(fallback)?.into_os_string()
            } else {
                return Err(ExpandError::EnvvarReadError(envvar.to_string()));
            };

            PathBuf::from(envvar_value)
        } else {
            PathBuf::from(comp)
        };
        expanded_comps.extend(path.components().map(|c| c.as_os_str().to_os_string()));
    }

    #[cfg(debug_assertions)]
    println!("comps={expanded_comps:?}");

    if let Some(front) = expanded_comps.front() {
        if front.as_os_str() == OsStr::new("~") {
            let home = BASE_DIRS.home_dir();
            expanded_comps.pop_front();
            for comp in PathBuf::from(home).components().rev() {
                expanded_comps.push_front(comp.as_os_str().to_os_string());
            }
        }
    }

    Ok(PathBuf::from_iter(expanded_comps))
}

#[cfg(test)]
mod tests {
    use anyhow::Context;

    use super::*;

    #[test]
    fn test_fails_to_expand_non_existent_envvar() {
        const TEST_ENVVAR: &str = "NO_WAY_YOU_HAVE_DEFINED_THIS";

        match expand(&format!("${TEST_ENVVAR}/some/file")) {
            Err(ExpandError::EnvvarReadError(envvar)) => assert_eq!(envvar, TEST_ENVVAR),
            res => panic!("expected error, got {res:?}"),
        }
    }

    #[test]
    fn test_parses_path_with_braces() {
        let expected = vec!["${within/braces}", "file"];
        let actual = __parse_path_components_with_braces("${within/braces}/file");

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_parses_path_with_braces_but_no_dollar_sign() {
        let expected = vec!["{within", "braces}", "file"];
        let actual = __parse_path_components_with_braces("{within/braces}/file");

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_expand_tilde() -> anyhow::Result<()> {
        let home = std::env::var("HOME").context("failed to get home dir")?;
        let expected = PathBuf::from(format!("{home}/path/to/file"));
        let actual = expand("~/path/to/file")?;

        assert_eq!(expected, actual);

        Ok(())
    }

    #[test]
    fn test_expand_envvar() -> anyhow::Result<()> {
        let home = std::env::var("HOME").context("failed to get home dir")?;
        let expected = PathBuf::from(format!("{home}/some/file"));
        let actual = expand("$HOME/some/file")?;

        assert_eq!(expected, actual);

        Ok(())
    }

    #[test]
    fn test_expand_envvar_in_middle() -> anyhow::Result<()> {
        const TEST_ENVVAR_KEY: &str = "__SHELLEXPAND_TEST_ENVVAR";
        const TEST_ENVVAR_VALUE: &str = "test_value";

        let test_envvar_value = unsafe {
            std::env::set_var(TEST_ENVVAR_KEY, "test_value");
            let value = std::env::var(TEST_ENVVAR_KEY).context("failed to set test envvar")?;
            assert_eq!(
                TEST_ENVVAR_VALUE, value,
                "failed to set {TEST_ENVVAR_KEY}={TEST_ENVVAR_VALUE}, got '{value}' instead."
            );

            value
        };

        let expected = PathBuf::from(format!("/path/to/{test_envvar_value}/some/file"));
        let actual = expand(&format!("/path/to/${TEST_ENVVAR_KEY}/some/file"))?;

        assert_eq!(expected, actual);

        Ok(())
    }

    #[test]
    fn test_expand_envvar_with_braces() -> anyhow::Result<()> {
        let home = std::env::var("HOME").context("failed to get home dir")?;
        let expected = PathBuf::from(format!("{home}/some/file"));
        let actual = expand("${HOME}/some/file")?;

        assert_eq!(expected, actual);

        Ok(())
    }

    #[test]
    fn test_expand_fallback_envvar() -> anyhow::Result<()> {
        let home = std::env::var("HOME").context("failed to get home dir")?;
        let expected = PathBuf::from(format!("{home}/some/file"));
        let actual = expand("${NO_WAY_YOU_HAVE_DEFINED_THIS:-$HOME}/some/file")?;

        assert_eq!(expected, actual);

        Ok(())
    }

    #[test]
    fn test_expand_nested_fallback_envvars() -> anyhow::Result<()> {
        let home = std::env::var("HOME").context("failed to get home dir")?;
        let expected = PathBuf::from(format!("{home}/some/file"));
        // braces are important! otherwise, it's ambiguous
        let actual = expand("${MISSING1:-${MISSING2:-${MISSING3:-$HOME}}}/some/file")?;

        assert_eq!(expected, actual);

        Ok(())
    }

    #[test]
    fn test_expand_fallback_tilde() -> anyhow::Result<()> {
        let home = std::env::var("HOME").context("failed to get home dir")?;
        let expected = PathBuf::from(format!("{home}/some/file"));
        let actual = expand("${NO_WAY_YOU_HAVE_DEFINED_THIS:-~}/some/file")?;

        assert_eq!(expected, actual);

        Ok(())
    }

    #[test]
    fn test_fallback_has_path_components() -> anyhow::Result<()> {
        let home = std::env::var("HOME").context("failed to get home dir")?;
        let expected = PathBuf::from(format!("{home}/some/file"));

        let actual = expand("${NO_WAY_YOU_HAVE_DEFINED_THIS:-~/some}/file")?;

        assert_eq!(expected, actual);

        Ok(())
    }
}
