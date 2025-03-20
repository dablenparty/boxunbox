use std::{path::PathBuf, sync::LazyLock};
use std::{
    collections::VecDeque,
    ffi::OsStr,
    path::{Component, PathBuf},
    sync::LazyLock,
};

use anyhow::Context;
use regex::Regex;

fn expand(s: &str) -> anyhow::Result<PathBuf> {
    static ENVVAR_REGEX: LazyLock<Regex> = LazyLock::new(|| {
        /*
         * Allowed syntax:
         * Vars must start with a $ and either a letter or underscore. This may be followed by any
         * amount of letters, numbers, or underscores. I'm not supporting anything else because if
         * you're doing something else, why? Fallback values may be defined after a `:-` (see
         * examples 3 & 4).
         *
         * Example matches:
         * 1. $ENV_VAR
         * 2. ${ENV_VAR}
         * 3. ${MISSING_VAR:-$ENV_VAR}
         * 4. ${MISSING_VAR:-~/path/to/file}
         *
         * There are three capture groups:
         * 1. The environment variable (minus the $)
         *    ([\w_][\w\d_]*)
         * 2. Everything after (ignore this group)
         *    (:-(.*)?.)?
         * 3. The fallback value
         *    (.*)?
         *
         * The extra dot after capture group 3 is required. Without it, group 3 picks up on the
         * closing brace since it's greedy. The extra dot bypasses that, but the explicit brace is
         * also required because... idk why. It might not be, but I want it there for brevity.
         */
        Regex::new(r"\$\{?([\w_][\w\d_]*)(:-(.*)?.)?\}?").expect("invalid envvar regex")
    });

    // TODO: thiserror errors
    let path = PathBuf::from(s);
    let comp_strs = path.components().map(|c| c.as_os_str()).collect::<Vec<_>>();
    let mut expanded_comps = Vec::with_capacity(comp_strs.len());

    for comp in comp_strs {
        let path = if let Some(captures) = ENVVAR_REGEX.captures(&comp.to_string_lossy()) {
            let envvar = captures
                .get(1)
                .and_then(|m| if m.is_empty() { None } else { Some(m.as_str()) })
                .context("matched envvar regex, but failed to capture envvar")?;

            #[cfg(debug_assertions)]
            println!("expanding envvar '{envvar:?}'");

            let envvar_value = match std::env::var_os(envvar) {
                Some(value) => value,
                None => {
                    if let Some(fallback) = captures.get(3) {
                        expand(fallback.as_str())?.into_os_string()
                    } else {
                        anyhow::bail!("failed to get value of var: {envvar:?}")
                    }
                }
            };

            #[cfg(debug_assertions)]
            println!("{envvar:?}={envvar_value:?}");

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
            #[cfg(unix)]
            let home = std::env::var_os("HOME").context("failed to get HOME")?;
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
}
