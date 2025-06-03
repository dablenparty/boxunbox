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

pub fn plan_unboxing(
    root_config: PackageConfig,
    cli: &BoxUnboxCli,
) -> Result<Vec<PlannedLink>, PlanningError> {
    let mut targets = Vec::new();
    let mut config_stack = vec![root_config];
    let mut walker = walkdir::WalkDir::new(config_stack[0].package.clone())
        .sort_by_file_name()
        .into_iter();
    // skip root
    let _ = walker.next();
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
            package,
            target,
            link_type,
            ..
        } = current_config;

        let path_tail = entry_path
            .strip_prefix(package)
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
}
