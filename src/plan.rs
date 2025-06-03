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

    use crate::{
        constants::BASE_DIRS,
        test_utils::{TEST_PACKAGE_FILE_TAILS, make_tmp_tree},
    };

    use super::*;

    #[test]
    fn test_plan_unboxing() -> anyhow::Result<()> {
        let package = make_tmp_tree().context("failed to make test package")?;
        let package_path = package.path();
        let cli = BoxUnboxCli::new(package_path);
        let config = PackageConfig::init(package_path, &cli)
            .context("failed to create test package config")?;

        let expected_plan = TEST_PACKAGE_FILE_TAILS
            .iter()
            .map(|tail| PlannedLink {
                src: package_path.join(tail),
                dest: BASE_DIRS.home_dir().join(tail),
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
        todo!()
    }
}
