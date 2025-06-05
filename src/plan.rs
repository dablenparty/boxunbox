use std::path::PathBuf;

use crate::{
    cli::BoxUnboxCli,
    error::PlanningError,
    package::{LinkType, PackageConfig, error::ConfigRead},
};

#[derive(Debug, PartialEq, Eq)]
pub struct PlannedLink {
    src: PathBuf,
    dest: PathBuf,
    ty: LinkType,
}

/// Plan an unboxing. This takes a [`PackageConfig`] and CLI and returns a list of
/// [`PlannedLink`]s.
///
/// # Arguments
///
/// - `root_config` - The initial config to plan with.
/// - `cli` - CLI flags to merge with [`PackageConfig`]s.
///
/// # Errors
///
/// An error is returned if one occurs while parsing nested [`PackageConfig`]s, converting old RON
/// configs to TOML, or walking the package tree.
pub fn plan_unboxing(
    root_config: PackageConfig,
    cli: &BoxUnboxCli,
) -> Result<Vec<PlannedLink>, PlanningError> {
    let mut targets = Vec::new();
    let mut config_stack = vec![root_config];
    let mut walker = walkdir::WalkDir::new(config_stack[0].package.clone())
        .sort_by_file_name()
        .into_iter();
    // don't process root package, although keep it for later
    let root_entry = walker.next().expect("walker should contain root entry")?;
    while let Some(res) = walker.next() {
        let entry = res?;
        let file_name = entry.file_name().to_string_lossy();
        let file_type = entry.file_type();
        let should_be_ignored = config_stack
            .iter()
            .flat_map(|conf| conf.ignore_pats.clone())
            .any(|re| re.is_match(&file_name));

        if should_be_ignored {
            if file_type.is_dir() {
                walker.skip_current_dir();
            }
            continue;
        }

        let entry_path = entry.path();
        let current_config = config_stack
            .last()
            .expect("config_stack should not be empty");
        // if we're not in the package defined by the current config, pop the config from the stack
        if !entry_path.starts_with(&current_config.package) {
            let _ = config_stack.pop().unwrap();
        }

        if file_type.is_dir() {
            // read nested config
            match PackageConfig::init(entry_path, cli) {
                Ok(config) => config_stack.push(config),
                Err(ConfigRead::FileNotFound(_)) => {}
                Err(err) => return Err(err.into()),
            }

            continue;
        }

        // ent is definitely a file at this point
        // shadow current_config in case a new one was added
        let current_config = config_stack
            .last()
            .expect("config_stack should not be empty");

        let PackageConfig {
            target, link_type, ..
        } = current_config;

        // don't use nested package to strip prefix or else the files will not be placed in
        // the correct target directory.
        // strip_prefix with root package: <target>/folder1/nested1.txt
        // strip_prefix with nested package: <target>/nested1.txt
        let path_tail = entry_path
            .strip_prefix(root_entry.path())
            .expect("entry_path should be prefixed by package");

        targets.push(PlannedLink {
            src: entry_path.to_path_buf(),
            dest: target.join(path_tail),
            ty: *link_type,
        });
    }

    Ok(targets)
}

#[cfg(test)]
mod tests {
    use anyhow::Context;
    use regex::Regex;

    use crate::test_utils::{TEST_PACKAGE_FILE_TAILS, TEST_TARGET, make_tmp_tree};

    use super::*;

    #[test]
    fn test_plan_unboxing() -> anyhow::Result<()> {
        let package = make_tmp_tree().context("failed to make test package")?;
        let package_path = package.path();
        let cli = BoxUnboxCli::new(package_path);
        let config = PackageConfig::init(package_path, &cli)
            .context("failed to create test package config")?;

        let expected_target = PathBuf::from(TEST_TARGET);
        let expected_plan = TEST_PACKAGE_FILE_TAILS
            .iter()
            .map(|tail| PlannedLink {
                src: package_path.join(tail),
                dest: expected_target.join(tail),
                ty: LinkType::SymlinkAbsolute,
            })
            .collect::<Vec<_>>();
        let actual_plan = plan_unboxing(config, &cli)?;

        assert_eq!(
            expected_plan.len(),
            actual_plan.len(),
            "unboxing plan has unexpected length"
        );

        for pl in &actual_plan {
            assert!(
                expected_plan.contains(pl),
                "unboxing plan contains unexpected planned link: {pl:?}"
            );
        }

        Ok(())
    }

    #[test]
    fn test_plan_unboxing_nested_config() -> anyhow::Result<()> {
        const TEST_NESTED_PACKAGE: &str = "folder1";

        let package = make_tmp_tree().context("failed to make test package")?;
        let package_path = package.path();
        let cli = BoxUnboxCli::new(package_path);
        let config = PackageConfig::init(package_path, &cli)
            .context("failed to create test package config")?;

        // make nested config
        // NOTE: don't use TEST_TARGET here, we want to make sure the target change works
        let expected_nested_target = PathBuf::from("/some/other/test/target");
        let expected_nested_package = package_path.join(TEST_NESTED_PACKAGE);
        let mut nested_config = PackageConfig::new_with_target(
            expected_nested_package.clone(),
            expected_nested_target.clone(),
        );
        nested_config
            .ignore_pats
            .push(Regex::new("^test_ignore.*").expect("test Regex should compile"));
        nested_config
            .save_to_package()
            .context("failed to save nested test config to test package")?;

        let expected_target = PathBuf::from(TEST_TARGET);
        let expected_plan = TEST_PACKAGE_FILE_TAILS
            .into_iter()
            .filter(|tail| !tail.ends_with("test_ignore2.txt"))
            .map(|tail| {
                let dest = if tail.starts_with(TEST_NESTED_PACKAGE) {
                    expected_nested_target.join(tail)
                } else {
                    expected_target.join(tail)
                };

                PlannedLink {
                    src: package_path.join(tail),
                    dest,
                    ty: LinkType::SymlinkAbsolute,
                }
            })
            .collect::<Vec<_>>();
        let actual_plan = plan_unboxing(config, &cli)?;

        assert_eq!(
            expected_plan.len(),
            actual_plan.len(),
            "unboxing plan has unexpected length"
        );

        for pl in &actual_plan {
            assert!(
                expected_plan.contains(pl),
                "unboxing plan contains unexpected planned link: {pl:?}"
            );

            if pl.src.starts_with(&expected_nested_package) {
                assert!(
                    pl.dest.starts_with(&expected_nested_target),
                    "nested planned link has unexpected target '{:?}'",
                    pl.dest
                );
            }
        }

        Ok(())
    }

    #[test]
    fn test_plan_unbox_root() -> anyhow::Result<()> {
        let package = make_tmp_tree().context("failed to make test package")?;
        let package_path = package.path();
        let mut cli = BoxUnboxCli::new(package_path);
        // just override with CLI; don't bother re-saving the package config
        cli.link_root = true;
        let config = PackageConfig::init(package_path, &cli)
            .context("failed to create test package config")?;

        let expected_target = PathBuf::from(TEST_TARGET);
        let expected_plan = [PlannedLink {
            src: package_path.to_path_buf(),
            dest: expected_target.clone(),
            ty: LinkType::SymlinkAbsolute,
        }];
        let actual_plan = plan_unboxing(config, &cli)?;

        assert_eq!(
            expected_plan.len(),
            actual_plan.len(),
            "unboxing plan has unexpected length"
        );

        for pl in &actual_plan {
            assert!(
                expected_plan.contains(pl),
                "unboxing plan contains unexpected planned link: {pl:?}"
            );
        }

        Ok(())
    }
}
