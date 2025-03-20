use std::path::PathBuf;

fn expand(s: &str) -> PathBuf {
    todo!("custom shellexpand")
}

#[cfg(test)]
mod tests {
    use anyhow::Context;

    use super::*;

    #[test]
    fn test_expand_tilde() -> anyhow::Result<()> {
        let home = std::env::var("HOME").context("failed to get home dir")?;
        let expected = PathBuf::from(format!("{home}/path/to/file"));
        let actual = expand("~/path/to/file");

        assert_eq!(expected, actual);

        Ok(())
    }

    #[test]
    fn test_expand_envvar() -> anyhow::Result<()> {
        let home = std::env::var("HOME").context("failed to get home dir")?;
        let expected = PathBuf::from(format!("{home}/some/file"));
        let actual = expand("$HOME/some/file");

        assert_eq!(expected, actual);

        Ok(())
    }

    #[test]
    fn test_expand_envvar_with_braces() -> anyhow::Result<()> {
        let home = std::env::var("HOME").context("failed to get home dir")?;
        let expected = PathBuf::from(format!("{home}/some/file"));
        let actual = expand("${HOME}/some/file");

        assert_eq!(expected, actual);

        Ok(())
    }
}
