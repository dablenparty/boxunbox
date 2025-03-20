use std::{path::PathBuf, sync::LazyLock};

use anyhow::Context;
use regex::Regex;

fn expand(s: &str) -> anyhow::Result<PathBuf> {
    static ENVVAR_REGEX: LazyLock<Regex> = LazyLock::new(|| {
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

    // TODO: expand ~
    Ok(PathBuf::from_iter(expanded_comps.iter()))
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
}
