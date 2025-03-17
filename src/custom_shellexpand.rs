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
        let expected = PathBuf::from(format!("/path/to{home}"));
        let actual = expand("/path/to/$HOME");

        assert_eq!(expected, actual);

        Ok(())
    }
}
