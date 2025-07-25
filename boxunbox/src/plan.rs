use std::{
    fmt::Display,
    fs, io,
    path::{Path, PathBuf},
};

use colored::Colorize;
use pathdiff::diff_paths;

use crate::{
    cli::{BoxUnboxCli, ExistingFileStrategy},
    error::{PlanningError, UnboxError},
    package::{LinkType, PackageConfig, error::ConfigRead},
    utils::{os_symlink, replace_home_with_tilde},
};

pub struct DisplayPlan<'a> {
    plan: &'a UnboxPlan,
    root_config: &'a PackageConfig,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PlannedLink {
    src: PathBuf,
    dest: PathBuf,
    ty: LinkType,
}

#[derive(Debug)]
pub struct UnboxPlan {
    /// Planned links
    links: Vec<PlannedLink>,
    /// What to do if [`PlannedLink::dest`] exists
    efs: ExistingFileStrategy,

    #[cfg(debug_assertions)]
    /// Whether to create missing dirs in `target` or not
    create_dirs: bool,
}

impl Display for DisplayPlan<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self { plan, root_config } = self;

        let UnboxPlan {
            links,
            efs,
            #[cfg(debug_assertions)]
            create_dirs,
        } = plan;

        let PackageConfig {
            package,
            target,
            link_root,
            ..
        } = root_config;

        writeln!(f, "Here's the unboxing plan:")?;
        writeln!(
            f,
            "Package: {}",
            replace_home_with_tilde(package).bright_green()
        )?;
        writeln!(f, "Target: {}", replace_home_with_tilde(target).cyan())?;

        // `link_root` means the target is linked directly to the package. In that case, stripping
        // the package/target prefix would return an empty string, so we don't strip if `link_root`
        // is `true`.
        let path_formatter = if *link_root {
            |p: &Path, _: &Path| replace_home_with_tilde(p)
        } else {
            |p: &Path, prefix: &Path| {
                p.strip_prefix(prefix).map_or_else(
                    |_| replace_home_with_tilde(p),
                    |stripped| stripped.to_string_lossy().to_string(),
                )
            }
        };

        let mut links = links.clone();
        links.sort_by_key(|pl| pl.dest.clone());

        for pl in &links {
            let PlannedLink { src, dest, ty } = pl;
            let formatted_dest = path_formatter(dest, target);
            let formatted_src = path_formatter(src, package);

            match ty {
                LinkType::SymlinkAbsolute => {
                    writeln!(
                        f,
                        "{} -> {}",
                        formatted_dest.cyan(),
                        formatted_src.bright_green(),
                    )?;
                }
                LinkType::SymlinkRelative => {
                    let relative_src = pl.get_src_relative_to_dest();
                    writeln!(
                        f,
                        "{} -> {}",
                        formatted_dest.cyan(),
                        relative_src.display().to_string().bright_green(),
                    )?;
                }
                LinkType::HardLink => {
                    writeln!(
                        f,
                        "{} ({}) -> {}",
                        formatted_dest.cyan(),
                        "hard link".bright_red(),
                        formatted_src.bright_green(),
                    )?;
                }
            }
        }

        write!(f, "If a target file already exists, it will ")?;

        let efs_verb = match efs {
            ExistingFileStrategy::Adopt => "be adopted".green(),
            ExistingFileStrategy::Ignore => "be ignored".cyan(),
            ExistingFileStrategy::Move => "be moved to <target_file>.bak".yellow(),
            ExistingFileStrategy::Overwrite => "be overwritten".bright_red(),
            ExistingFileStrategy::ThrowError => "throw an error".bright_red(),
        };

        writeln!(f, "{efs_verb}")?;

        #[cfg(debug_assertions)]
        {
            write!(f, "{}: Target directories will ", "debug".cyan())?;
            let create_dirs_verb = if *create_dirs {
                "be created".cyan()
            } else {
                "not be created".bright_red()
            };

            writeln!(f, "{create_dirs_verb}")?;
        }

        Ok(())
    }
}

#[cfg(test)]
impl<A: Into<PlannedLink>> FromIterator<A> for UnboxPlan {
    fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> Self {
        Self {
            links: iter.into_iter().map(Into::into).collect(),
            efs: ExistingFileStrategy::default(),

            #[cfg(debug_assertions)]
            create_dirs: true,
        }
    }
}

impl PlannedLink {
    /// Utility function that returns a modified [`PlannedLink::src`] that is relative to the
    /// parent of [`PlannedLink::dest`]. Both paths must be absolute before calling this function.
    ///
    /// # Panics
    ///
    /// This function will panic if either `src` or `dest` are not absolute.
    fn get_src_relative_to_dest(&self) -> PathBuf {
        let Self { src, dest, .. } = self;

        assert!(src.is_absolute(), "{} is not absolute", src.display());
        assert!(dest.is_absolute(), "{} is not absolute", dest.display());

        let dest_parent = dest
            .parent()
            .expect("dest should be a file and have a parent");

        #[cfg(debug_assertions)]
        println!("diffing {} with {}", src.display(), dest_parent.display());

        diff_paths(src, dest_parent).expect("diff_paths should not return None")
    }

    /// Unbox this [`PlannedLink`] by creating either a symbolic link or a hard link, depending on
    /// [`Self::ty`].
    ///
    /// # Errors
    ///
    /// An error will be returned if the `dest` parent cannot be created, if [`Self::dest`] is not
    /// absolute, if [`os_symlink`] fails to create a symbolic link, or if [`fs::hard_link`] fails
    /// to create a hard link.
    pub fn unbox(&self, create_dirs: bool) -> io::Result<()> {
        let Self { src, dest, ty } = self;

        if create_dirs {
            let target_parent = dest.parent().ok_or(io::ErrorKind::InvalidFilename)?;
            fs::create_dir_all(target_parent)?;
        }

        match ty {
            LinkType::SymlinkAbsolute => {
                os_symlink(src, dest)?;
            }
            LinkType::SymlinkRelative => {
                let relative_src = self.get_src_relative_to_dest();
                os_symlink(relative_src, dest)?;
            }
            LinkType::HardLink => {
                fs::hard_link(src, dest)?;
            }
        }

        Ok(())
    }
}

impl UnboxPlan {
    /// Returns an object implementing [`Display`] for printing this [`UnboxPlan`] with
    /// supplemental information from a [`PackageConfig`]. This is modeled after
    /// [`std::path::Path::display`].
    ///
    /// # Arguments
    ///
    /// - `root_config` - [`PackageConfig`] to borrow info from.
    #[must_use]
    pub fn display<'a>(&'a self, root_config: &'a PackageConfig) -> DisplayPlan<'a> {
        DisplayPlan {
            plan: self,
            root_config,
        }
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
    ///
    /// # Panics
    ///
    /// This function will panic in the following cases:
    /// - It cannot find the package from `root_config`
    /// - It runs out of configs
    /// - A file NOT from the package is encountered
    pub fn plan_unboxing(
        root_config: PackageConfig,
        cli: &BoxUnboxCli,
    ) -> Result<Self, PlanningError> {
        let mut plan = Self {
            links: Vec::new(),
            efs: cli.existing_file_strategy,
            #[cfg(debug_assertions)]
            create_dirs: !cli.no_create_dirs,
        };

        if root_config.link_root {
            plan.links.push(PlannedLink {
                src: root_config.package,
                dest: root_config.target,
                ty: root_config.link_type,
            });
            // root_config should already be merged with cli
            return Ok(plan);
        }
        let targets = &mut plan.links;
        let mut config_stack = vec![root_config];
        let mut walker = walkdir::WalkDir::new(config_stack[0].package.clone())
            .sort_by_file_name()
            .into_iter();
        // don't process root entry
        let _root_entry = walker.next().expect("walker should contain root entry")?;
        while let Some(res) = walker.next() {
            let entry = res?;
            let file_name = entry.file_name().to_string_lossy();
            let file_type = entry.file_type();
            // Components ARE NOT needed for excluding because `walkdir` provides the handy dandy
            // `skip_current_dir` function (see below).
            let should_be_excluded = config_stack
                .iter()
                .flat_map(|conf| &conf.exclude_pats)
                .any(|re| re.is_match(&file_name));

            if should_be_excluded {
                // skips the current dir by removing it and all children from the iterator
                if file_type.is_dir() {
                    walker.skip_current_dir();
                }
                continue;
            }

            // NOTE: `include_pats` must be checked after `exclude_pats`.
            // If a directory is not explicitly included or excluded, it should be considered
            // included because it might have nested dirs or files that WILL match a pattern.
            // Therefore, the dir is skipped, but not removed from the iterator like above.
            let should_be_included = {
                let pats = config_stack
                    .iter()
                    .flat_map(|conf| &conf.include_pats)
                    .collect::<Vec<_>>();
                // Components ARE needed for including or else nothing will be included. For
                // example, a file may not match the include pattern but it's parent folder does,
                // or vice versa.
                let entry_components = entry
                    .path()
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy())
                    .collect::<Vec<_>>();
                pats.is_empty()
                    || pats
                        .iter()
                        .any(|re| entry_components.iter().any(|c| re.is_match(c)))
            };

            if !should_be_included {
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

            // entry is definitely a file at this point
            // shadow current_config in case a new one was added
            let current_config = config_stack
                .last()
                .expect("config_stack should not be empty");

            let PackageConfig {
                target,
                link_type,
                package: current_package,
                ..
            } = current_config;

            let path_tail = entry_path
                .strip_prefix(current_package)
                .expect("entry_path should be prefixed by package");

            targets.push(PlannedLink {
                src: entry_path.to_path_buf(),
                dest: target.join(path_tail),
                ty: *link_type,
            });
        }

        if plan.links.is_empty() {
            Err(PlanningError::EmptyPlan)
        } else {
            Ok(plan)
        }
    }

    /// Unbox the package according to this [`UnboxPlan`], handling any existing target files along
    /// the way.
    ///
    /// # Errors
    ///
    /// An error will be returned if:
    /// - When using [`ExistingFileStrategy::Adopt`], the target file is a symlink, or cannot be
    ///   copied to the package, or cannot be removed after.
    /// - When using [`ExistingFileStrategy::Move`], the target file cannot be moved.
    /// - When using [`ExistingFileStrategy::Overwrite`], the target file cannot be removed.
    /// - In any case, [`PlannedLink::unbox`] returns an error.
    ///
    /// # Panics
    ///
    /// This function will panic if a file name cannot be retrieved from a [`PlannedLink`]. Their
    /// `src` and `dest` fields are expected to be absolute paths.
    pub fn unbox(&self) -> Result<(), UnboxError> {
        for pl in &self.links {
            let PlannedLink { src, dest, .. } = &pl;

            if dest.try_exists().map_err(|err| UnboxError::Io {
                pl: pl.clone(),
                source: err,
            })? {
                // TODO: put messages behind --verbose flag (idk how to go about this)
                match self.efs {
                    ExistingFileStrategy::Adopt if !dest.is_symlink() => {
                        // If dest is a symlink, it might point to the src file, which means we'd
                        // be copying a file into itself, thus truncating it. Not ideal.
                        eprintln!(
                            "{}: adopting {}",
                            "warn".yellow(),
                            replace_home_with_tilde(dest)
                        );
                        fs::copy(dest, src).map_err(|err| UnboxError::Io {
                            pl: pl.clone(),
                            source: err,
                        })?;
                        // remove `dest` so that it can be replaced by a link
                        fs::remove_file(dest).map_err(|err| UnboxError::Io {
                            pl: pl.clone(),
                            source: err,
                        })?;
                    }
                    ExistingFileStrategy::Adopt => {
                        // TODO: force adopt symlink with CLI flag?
                        return Err(UnboxError::AdoptSymlink(pl.clone()));
                    }
                    ExistingFileStrategy::Ignore => {
                        eprintln!(
                            "{}: ignoring {} (already exists)",
                            "warn".yellow(),
                            replace_home_with_tilde(dest)
                        );
                        continue;
                    }
                    ExistingFileStrategy::Move => {
                        let file_name = dest
                            .file_name()
                            .expect("dest should be an absolute path to a file")
                            .to_string_lossy();
                        let new_dest = dest.with_file_name(format!("{file_name}.bak"));
                        eprintln!(
                            "{}: dest exists, moving {} -> {}",
                            "warn".yellow(),
                            replace_home_with_tilde(dest),
                            replace_home_with_tilde(&new_dest)
                        );
                        fs::rename(dest, new_dest).map_err(|err| UnboxError::Io {
                            pl: pl.clone(),
                            source: err,
                        })?;
                    }
                    ExistingFileStrategy::Overwrite => {
                        eprintln!(
                            "{}: overwriting {}",
                            "warn".yellow(),
                            replace_home_with_tilde(dest)
                        );
                        fs::remove_file(dest).map_err(|err| UnboxError::Io {
                            pl: pl.clone(),
                            source: err,
                        })?;
                    }
                    ExistingFileStrategy::ThrowError => {
                        return Err(UnboxError::TargetAlreadyExists(pl.clone()));
                    }
                }
            }

            #[cfg(debug_assertions)]
            let create_dirs = self.create_dirs;

            #[cfg(not(debug_assertions))]
            let create_dirs = true;

            pl.unbox(create_dirs).map_err(|err| UnboxError::Io {
                pl: pl.clone(),
                source: err,
            })?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use anyhow::Context;
    use regex::Regex;

    use crate::test_utils::{TEST_PACKAGE_FILE_TAILS, TEST_TARGET, make_tmp_tree};

    use super::*;

    #[test]
    fn test_unbox_default() -> anyhow::Result<()> {
        let package = make_tmp_tree().context("failed to make test package")?;
        let package_path = package.path();

        let target = tempfile::tempdir().context("failed to create temp target")?;
        let target_path = target.path();

        let expected_target = PathBuf::from(target_path);
        let expected_plan = TEST_PACKAGE_FILE_TAILS
            .iter()
            .map(|tail| PlannedLink {
                src: package_path.join(tail),
                dest: expected_target.join(tail),
                ty: LinkType::default(),
            })
            .collect::<UnboxPlan>();

        assert_eq!(
            expected_plan.efs,
            ExistingFileStrategy::default(),
            "unboxing plan has unexpected {}",
            stringify!(ExistingFileStrategy)
        );

        expected_plan.unbox()?;

        for link in expected_plan.links {
            let PlannedLink { src, dest, .. } = link;

            assert!(
                dest.try_exists()
                    .with_context(|| format!("failed to verify existence of {}", dest.display()))?,
                "{} does not exist",
                dest.display()
            );
            assert!(dest.is_symlink(), "expected symlink at {}", dest.display());
            let actual_link_target = fs::read_link(&dest)
                .with_context(|| format!("failed to read link info for {}", dest.display()))?;
            assert_eq!(
                src,
                actual_link_target,
                "{} does not point to {}",
                actual_link_target.display(),
                src.display()
            );
        }

        Ok(())
    }

    #[test]
    fn test_unbox_package_symlink() -> anyhow::Result<()> {
        let package = make_tmp_tree().context("failed to make test package")?;
        let package_path = package.path();

        let target = tempfile::tempdir().context("failed to create temp target")?;
        let target_path = target.path();

        let expected_target = PathBuf::from(target_path);
        // only test with one file
        let tail = TEST_PACKAGE_FILE_TAILS[0];
        let expected_planned_link = PlannedLink {
            src: package_path.join(tail),
            dest: expected_target.join(tail),
            ty: LinkType::default(),
        };
        let expected_plan = TEST_PACKAGE_FILE_TAILS
            .iter()
            .map(|tail| PlannedLink {
                src: package_path.join(tail),
                dest: expected_target.join(tail),
                ty: LinkType::default(),
            })
            .collect::<UnboxPlan>();

        let link_tail = TEST_PACKAGE_FILE_TAILS[1];
        fs::remove_file(&expected_planned_link.src)?;
        os_symlink(package_path.join(link_tail), &expected_planned_link.src)
            .context("failed to create test symlink")?;

        assert!(
            expected_planned_link.src.is_symlink(),
            "expected symlink at {}",
            expected_planned_link.src.display()
        );

        assert_eq!(
            expected_plan.efs,
            ExistingFileStrategy::default(),
            "unboxing plan has unexpected {}",
            stringify!(ExistingFileStrategy)
        );

        expected_plan.unbox()?;

        for link in expected_plan.links {
            let PlannedLink { src, dest, .. } = link;

            assert!(
                dest.try_exists()
                    .with_context(|| format!("failed to verify existence of {}", dest.display()))?,
                "{} does not exist",
                dest.display()
            );
            assert!(dest.is_symlink(), "expected symlink at {}", dest.display());
            let actual_link_target = fs::read_link(&dest)
                .with_context(|| format!("failed to read link info for {}", dest.display()))?;
            assert_eq!(
                src,
                actual_link_target,
                "{} does not point to {}",
                actual_link_target.display(),
                src.display()
            );
        }

        Ok(())
    }

    #[test]
    fn test_unbox_absolute_symlinks() -> anyhow::Result<()> {
        // NOTE: this test is kinda a repeat of test_unbox_default, but if I ever change the
        let package = make_tmp_tree().context("failed to make test package")?;
        let package_path = package.path();

        let target = tempfile::tempdir().context("failed to create temp target")?;
        let target_path = target.path();

        let expected_target = PathBuf::from(target_path);
        let expected_plan = TEST_PACKAGE_FILE_TAILS
            .iter()
            .map(|tail| PlannedLink {
                src: package_path.join(tail),
                dest: expected_target.join(tail),
                ty: LinkType::SymlinkAbsolute,
            })
            .collect::<UnboxPlan>();

        assert_eq!(
            expected_plan.efs,
            ExistingFileStrategy::ThrowError,
            "unboxing plan has unexpected {}",
            stringify!(ExistingFileStrategy)
        );

        expected_plan
            .unbox()
            .context("failed to unbox test package")?;

        for link in expected_plan.links {
            let PlannedLink { src, dest, .. } = link;

            assert!(
                dest.try_exists()
                    .with_context(|| format!("failed to verify existence of {}", dest.display()))?,
                "{} does not exist",
                dest.display()
            );
            assert!(dest.is_symlink(), "expected symlink at {}", dest.display());
            let actual_link_target = fs::read_link(&dest)
                .with_context(|| format!("failed to read link info for {}", dest.display()))?;
            assert_eq!(
                src,
                actual_link_target,
                "{} does not point to {}",
                actual_link_target.display(),
                src.display()
            );
        }

        Ok(())
    }

    #[test]
    fn test_unbox_relative_symlinks() -> anyhow::Result<()> {
        // NOTE: this test is kinda a repeat of test_unbox_default, but if I ever change the
        let package = make_tmp_tree().context("failed to make test package")?;
        let package_path = package.path();

        let target = tempfile::tempdir().context("failed to create temp target")?;
        let target_path = target.path();

        let expected_target = PathBuf::from(target_path);
        let expected_plan = TEST_PACKAGE_FILE_TAILS
            .iter()
            .map(|tail| PlannedLink {
                src: package_path.join(tail),
                dest: expected_target.join(tail),
                ty: LinkType::SymlinkRelative,
            })
            .collect::<UnboxPlan>();

        assert_eq!(
            expected_plan.efs,
            ExistingFileStrategy::ThrowError,
            "unboxing plan has unexpected {}",
            stringify!(ExistingFileStrategy)
        );

        expected_plan
            .unbox()
            .context("failed to unbox test package")?;

        for link in expected_plan.links {
            let relative_src = link.get_src_relative_to_dest();
            let PlannedLink { dest, .. } = link;

            assert!(
                dest.try_exists()
                    .with_context(|| format!("failed to verify existence of {}", dest.display()))?,
                "{} does not exist",
                dest.display()
            );
            assert!(dest.is_symlink(), "expected symlink at {}", dest.display());
            let actual_link_target = fs::read_link(&dest)
                .with_context(|| format!("failed to read link info for {}", dest.display()))?;
            assert_eq!(
                relative_src,
                actual_link_target,
                "{} does not point to {}",
                actual_link_target.display(),
                relative_src.display()
            );
        }

        Ok(())
    }

    #[test]
    fn test_unbox_hard_links() -> anyhow::Result<()> {
        // NOTE: this test is kinda a repeat of test_unbox_default, but if I ever change the
        let package = make_tmp_tree().context("failed to make test package")?;
        let package_path = package.path();

        let target = tempfile::tempdir().context("failed to create temp target")?;
        let target_path = target.path();

        let expected_target = PathBuf::from(target_path);
        let expected_plan = TEST_PACKAGE_FILE_TAILS
            .iter()
            .map(|tail| PlannedLink {
                src: package_path.join(tail),
                dest: expected_target.join(tail),
                ty: LinkType::HardLink,
            })
            .collect::<UnboxPlan>();

        assert_eq!(
            expected_plan.efs,
            ExistingFileStrategy::ThrowError,
            "unboxing plan has unexpected {}",
            stringify!(ExistingFileStrategy)
        );

        expected_plan
            .unbox()
            .context("failed to unbox test package")?;

        for link in expected_plan.links {
            let PlannedLink { src, dest, .. } = link;

            assert!(
                dest.try_exists()
                    .with_context(|| format!("failed to verify existence of {}", dest.display()))?,
                "{} does not exist",
                dest.display()
            );
            assert!(
                dest.is_file(),
                "expected hard link (i.e. file) at {}",
                dest.display()
            );
            let src_contents = fs::read_to_string(&src)
                .with_context(|| format!("failed to read test src {}", src.display()))?;
            let dest_contents = fs::read_to_string(&dest)
                .with_context(|| format!("failed to read test dest {}", dest.display()))?;
            assert_eq!(
                src_contents, dest_contents,
                "target hard link has unexpected file contents '{dest_contents:?}'"
            );
        }

        Ok(())
    }

    #[test]
    fn test_unbox_efs_adopt_file() -> anyhow::Result<()> {
        const EXISTING_TARGET_FILE_CONTENTS: &str = "i already exist";

        let package = make_tmp_tree().context("failed to make test package")?;
        let package_path = package.path();

        let target = tempfile::tempdir().context("failed to create temp target")?;
        let target_path = target.path();
        let expected_target = PathBuf::from(target_path);

        let test_file_tail = TEST_PACKAGE_FILE_TAILS[0];
        let expected_pl = PlannedLink {
            src: package_path.join(test_file_tail),
            dest: expected_target.join(test_file_tail),
            ty: LinkType::SymlinkAbsolute,
        };
        // create the file in it's own scope to close file descriptor asap
        {
            let parent = expected_pl.dest.parent().with_context(|| {
                format!("failed to get parent of {}", expected_pl.dest.display())
            })?;
            fs::create_dir_all(parent).context("failed to create test target parent")?;
            fs::File::create_new(&expected_pl.dest)
                .context("failed to create test target file")?
                .write_all(EXISTING_TARGET_FILE_CONTENTS.as_bytes())?;
        }
        let mut expected_plan = TEST_PACKAGE_FILE_TAILS
            .iter()
            .map(|tail| PlannedLink {
                src: package_path.join(tail),
                dest: expected_target.join(tail),
                ty: LinkType::SymlinkAbsolute,
            })
            .collect::<UnboxPlan>();
        expected_plan.efs = ExistingFileStrategy::Adopt;

        let unbox_result = expected_plan.unbox();
        assert!(
            unbox_result.is_ok(),
            "unboxing failed with unexpected error {:?}",
            unbox_result.unwrap_err()
        );

        for link in expected_plan.links {
            if link == expected_pl {
                // src needs to match too
                let src_contents =
                    fs::read_to_string(&link.src).context("failed to read test src file")?;
                assert_eq!(
                    EXISTING_TARGET_FILE_CONTENTS, src_contents,
                    "test src file has unexpected file contents '{src_contents:?}'"
                );
                let dest_contents = fs::read_to_string(&link.dest)
                    .context("failed to read existing target file")?;
                // proves src == dest by transitivity
                assert_eq!(
                    EXISTING_TARGET_FILE_CONTENTS, dest_contents,
                    "existing target file has unexpected file contents '{dest_contents:?}'"
                );
                continue;
            }

            let PlannedLink { src, dest, .. } = link;

            assert!(
                dest.try_exists()
                    .with_context(|| format!("failed to verify existence of {}", dest.display()))?,
                "{} exists",
                dest.display()
            );
            assert!(dest.is_symlink(), "expected symlink at {}", dest.display());
            let actual_link_target = fs::read_link(&dest)
                .with_context(|| format!("failed to read link info for {}", dest.display()))?;
            assert_eq!(
                src,
                actual_link_target,
                "{} does not point to {}",
                actual_link_target.display(),
                src.display()
            );
        }

        Ok(())
    }

    #[test]
    fn test_unbox_efs_adopt_symlink() -> anyhow::Result<()> {
        const EXISTING_TARGET_FILE_CONTENTS: &str = "i already exist";

        let package = make_tmp_tree().context("failed to make test package")?;
        let package_path = package.path();

        let target = tempfile::tempdir().context("failed to create temp target")?;
        let target_path = target.path();
        let expected_target = PathBuf::from(target_path);

        let test_file_tail = TEST_PACKAGE_FILE_TAILS[0];
        let expected_pl = PlannedLink {
            src: package_path.join(test_file_tail),
            dest: expected_target.join(test_file_tail),
            ty: LinkType::SymlinkAbsolute,
        };
        // create the file in it's own scope to close file descriptor asap
        {
            let parent = expected_pl.dest.parent().with_context(|| {
                format!("failed to get parent of {}", expected_pl.dest.display())
            })?;
            fs::create_dir_all(parent).context("failed to create test target parent")?;
            os_symlink(&expected_pl.src, &expected_pl.dest)
                .context("failed to create test symlink")?;
        }
        let mut expected_plan = TEST_PACKAGE_FILE_TAILS
            .iter()
            .map(|tail| PlannedLink {
                src: package_path.join(tail),
                dest: expected_target.join(tail),
                ty: LinkType::SymlinkAbsolute,
            })
            .collect::<UnboxPlan>();
        expected_plan.efs = ExistingFileStrategy::Adopt;

        let unbox_result = expected_plan.unbox();
        assert!(unbox_result.is_err(), "unboxing succeeded unexpectedly");

        match unbox_result.unwrap_err() {
            UnboxError::AdoptSymlink(actual_pl) => {
                assert_eq!(
                    expected_pl, actual_pl,
                    "unboxing failed to adopt unexepected link {actual_pl:?}"
                );
            }
            err => panic!("unboxing failed with unexpected error: {err:?}"),
        }

        for link in expected_plan.links {
            if link == expected_pl {
                let src_contents =
                    fs::read_to_string(&link.src).context("failed to read test src file")?;
                assert_ne!(
                    EXISTING_TARGET_FILE_CONTENTS, src_contents,
                    "test src file has unexpected file contents '{src_contents:?}'"
                );
                continue;
            }

            let PlannedLink { dest, .. } = link;

            assert!(
                !dest
                    .try_exists()
                    .with_context(|| format!("failed to verify existence of {}", dest.display()))?,
                "{} exists",
                dest.display()
            );
        }

        Ok(())
    }

    #[test]
    fn test_unbox_efs_ignore() -> anyhow::Result<()> {
        const EXISTING_TARGET_FILE_CONTENTS: &str = "i already exist";

        let package = make_tmp_tree().context("failed to make test package")?;
        let package_path = package.path();

        let target = tempfile::tempdir().context("failed to create temp target")?;
        let target_path = target.path();
        let expected_target = PathBuf::from(target_path);

        let test_file_tail = TEST_PACKAGE_FILE_TAILS[0];
        let expected_pl = PlannedLink {
            src: package_path.join(test_file_tail),
            dest: expected_target.join(test_file_tail),
            ty: LinkType::SymlinkAbsolute,
        };
        // create the file in it's own scope to close file descriptor asap
        {
            let parent = expected_pl.dest.parent().with_context(|| {
                format!("failed to get parent of {}", expected_pl.dest.display())
            })?;
            fs::create_dir_all(parent).context("failed to create test target parent")?;
            fs::File::create_new(&expected_pl.dest)
                .context("failed to create test target file")?
                .write_all(EXISTING_TARGET_FILE_CONTENTS.as_bytes())?;
        }
        let mut expected_plan = TEST_PACKAGE_FILE_TAILS
            .iter()
            .map(|tail| PlannedLink {
                src: package_path.join(tail),
                dest: expected_target.join(tail),
                ty: LinkType::SymlinkAbsolute,
            })
            .collect::<UnboxPlan>();
        expected_plan.efs = ExistingFileStrategy::Ignore;

        let unbox_result = expected_plan.unbox();
        assert!(
            unbox_result.is_ok(),
            "unboxing failed with unexpected error {:?}",
            unbox_result.unwrap_err()
        );

        for link in expected_plan.links {
            // this one is already handled; skip it
            if link == expected_pl {
                let src_contents =
                    fs::read_to_string(&link.src).context("failed to read test src file")?;
                assert_ne!(
                    EXISTING_TARGET_FILE_CONTENTS, src_contents,
                    "test src file has unexpected file contents '{src_contents:?}'"
                );
                let dest_contents = fs::read_to_string(&link.dest)
                    .context("failed to read existing target file")?;
                assert_eq!(
                    EXISTING_TARGET_FILE_CONTENTS, dest_contents,
                    "existing target file has unexpected file contents '{dest_contents:?}'"
                );
                continue;
            }

            let PlannedLink { src, dest, .. } = link;

            assert!(
                dest.try_exists()
                    .with_context(|| format!("failed to verify existence of {}", dest.display()))?,
                "{} exists",
                dest.display()
            );
            assert!(dest.is_symlink(), "expected symlink at {}", dest.display());
            let actual_link_target = fs::read_link(&dest)
                .with_context(|| format!("failed to read link info for {}", dest.display()))?;
            assert_eq!(
                src,
                actual_link_target,
                "{} does not point to {}",
                actual_link_target.display(),
                src.display()
            );
        }

        Ok(())
    }

    #[test]
    fn test_unbox_efs_move() -> anyhow::Result<()> {
        const EXISTING_TARGET_FILE_CONTENTS: &str = "i already exist";

        let package = make_tmp_tree().context("failed to make test package")?;
        let package_path = package.path();

        let target = tempfile::tempdir().context("failed to create temp target")?;
        let target_path = target.path();
        let expected_target = PathBuf::from(target_path);

        let test_file_tail = TEST_PACKAGE_FILE_TAILS[0];
        let expected_pl = PlannedLink {
            src: package_path.join(test_file_tail),
            dest: expected_target.join(test_file_tail),
            ty: LinkType::SymlinkAbsolute,
        };
        // create the file in it's own scope to close file descriptor asap
        {
            let parent = expected_pl.dest.parent().with_context(|| {
                format!("failed to get parent of {}", expected_pl.dest.display())
            })?;
            fs::create_dir_all(parent).context("failed to create test target parent")?;
            fs::File::create_new(&expected_pl.dest)
                .context("failed to create test target file")?
                .write_all(EXISTING_TARGET_FILE_CONTENTS.as_bytes())?;
        }
        let mut expected_plan = TEST_PACKAGE_FILE_TAILS
            .iter()
            .map(|tail| PlannedLink {
                src: package_path.join(tail),
                dest: expected_target.join(tail),
                ty: LinkType::SymlinkAbsolute,
            })
            .collect::<UnboxPlan>();
        expected_plan.efs = ExistingFileStrategy::Move;

        let unbox_result = expected_plan.unbox();
        assert!(
            unbox_result.is_ok(),
            "unboxing failed with unexpected error {:?}",
            unbox_result.unwrap_err()
        );

        for link in expected_plan.links {
            if link == expected_pl {
                let src_contents =
                    fs::read_to_string(&link.src).context("failed to read test src file")?;
                assert_ne!(
                    EXISTING_TARGET_FILE_CONTENTS, src_contents,
                    "test src file has unexpected file contents '{src_contents:?}'"
                );

                let moved_dest_file_name = link
                    .dest
                    .file_name()
                    .expect("test dest path should have a file name")
                    .to_string_lossy();
                let moved_dest = link
                    .dest
                    .with_file_name(format!("{moved_dest_file_name}.bak"));
                assert!(
                    moved_dest.try_exists().with_context(|| format!(
                        "failed to verify existence of moved target file {}",
                        moved_dest.display()
                    ))?,
                    "unboxing failed to create moved target file {}",
                    moved_dest.display()
                );

                let moved_dest_contents = fs::read_to_string(&moved_dest)
                    .context("failed to read existing target file")?;
                assert_eq!(
                    EXISTING_TARGET_FILE_CONTENTS, moved_dest_contents,
                    "moved target file has unexpected file contents '{moved_dest_contents:?}'"
                );

                // NOTE: most other tests have a `continue` here; this test does not because the
                // `dest` link still needs to be checked normally.
            }

            let PlannedLink { src, dest, .. } = link;

            assert!(
                dest.try_exists()
                    .with_context(|| format!("failed to verify existence of {}", dest.display()))?,
                "{} exists",
                dest.display()
            );
            assert!(dest.is_symlink(), "expected symlink at {}", dest.display());
            let actual_link_target = fs::read_link(&dest)
                .with_context(|| format!("failed to read link info for {}", dest.display()))?;
            assert_eq!(
                src,
                actual_link_target,
                "{} does not point to {}",
                actual_link_target.display(),
                src.display()
            );
        }

        Ok(())
    }

    #[test]
    fn test_unbox_efs_overwrite() -> anyhow::Result<()> {
        const EXISTING_TARGET_FILE_CONTENTS: &str = "i already exist";

        let package = make_tmp_tree().context("failed to make test package")?;
        let package_path = package.path();

        let target = tempfile::tempdir().context("failed to create temp target")?;
        let target_path = target.path();
        let expected_target = PathBuf::from(target_path);

        let test_file_tail = TEST_PACKAGE_FILE_TAILS[0];
        let expected_pl = PlannedLink {
            src: package_path.join(test_file_tail),
            dest: expected_target.join(test_file_tail),
            ty: LinkType::SymlinkAbsolute,
        };
        // create the file in it's own scope to close file descriptor asap
        {
            let parent = expected_pl.dest.parent().with_context(|| {
                format!("failed to get parent of {}", expected_pl.dest.display())
            })?;
            fs::create_dir_all(parent).context("failed to create test target parent")?;
            fs::File::create_new(&expected_pl.dest)
                .context("failed to create test target file")?
                .write_all(EXISTING_TARGET_FILE_CONTENTS.as_bytes())?;
        }
        let mut expected_plan = TEST_PACKAGE_FILE_TAILS
            .iter()
            .map(|tail| PlannedLink {
                src: package_path.join(tail),
                dest: expected_target.join(tail),
                ty: LinkType::SymlinkAbsolute,
            })
            .collect::<UnboxPlan>();
        expected_plan.efs = ExistingFileStrategy::Overwrite;

        let unbox_result = expected_plan.unbox();
        assert!(
            unbox_result.is_ok(),
            "unboxing failed with unexpected error {:?}",
            unbox_result.unwrap_err()
        );

        for link in expected_plan.links {
            if link == expected_pl {
                let src_contents =
                    fs::read_to_string(&link.src).context("failed to read test src file")?;
                assert_ne!(
                    EXISTING_TARGET_FILE_CONTENTS, src_contents,
                    "test src file has unexpected file contents '{src_contents:?}'"
                );
                // NOTE: most other tests have a `continue` here; this test does not because the
                // `dest` link still needs to be checked normally.
            }

            let PlannedLink { src, dest, .. } = link;

            assert!(
                dest.try_exists()
                    .with_context(|| format!("failed to verify existence of {}", dest.display()))?,
                "{} exists",
                dest.display()
            );
            assert!(dest.is_symlink(), "expected symlink at {}", dest.display());
            let actual_link_target = fs::read_link(&dest)
                .with_context(|| format!("failed to read link info for {}", dest.display()))?;
            assert_eq!(
                src,
                actual_link_target,
                "{} does not point to {}",
                actual_link_target.display(),
                src.display()
            );
        }

        Ok(())
    }

    #[test]
    fn test_unbox_efs_throwerror() -> anyhow::Result<()> {
        let package = make_tmp_tree().context("failed to make test package")?;
        let package_path = package.path();

        let target = tempfile::tempdir().context("failed to create temp target")?;
        let target_path = target.path();
        let expected_target = PathBuf::from(target_path);

        let test_file_tail = TEST_PACKAGE_FILE_TAILS[0];
        let expected_pl = PlannedLink {
            src: package_path.join(test_file_tail),
            dest: expected_target.join(test_file_tail),
            ty: LinkType::SymlinkAbsolute,
        };
        // create the file in it's own scope to close file descriptor asap
        {
            let parent = expected_pl.dest.parent().with_context(|| {
                format!("failed to get parent of {}", expected_pl.dest.display())
            })?;
            fs::create_dir_all(parent).context("failed to create test target parent")?;
            let _ = fs::File::create_new(&expected_pl.dest)
                .context("failed to create test target file")?;
        }
        let expected_plan = TEST_PACKAGE_FILE_TAILS
            .iter()
            .map(|tail| PlannedLink {
                src: package_path.join(tail),
                dest: expected_target.join(tail),
                ty: LinkType::SymlinkAbsolute,
            })
            .collect::<UnboxPlan>();

        let unbox_result = expected_plan.unbox();
        assert!(unbox_result.is_err(), "unboxing succeeded unexpectedly");
        match unbox_result.unwrap_err() {
            UnboxError::TargetAlreadyExists(actual_pl) => {
                assert_eq!(
                    expected_pl, actual_pl,
                    "unexpected file already exists: {actual_pl:?}",
                );
            }
            err => panic!("unboxing failed with unexpected error {err:?}"),
        }

        for link in expected_plan.links {
            // this one is already handled; skip it
            if link == expected_pl {
                continue;
            }

            let PlannedLink { dest, .. } = link;

            assert!(
                !dest
                    .try_exists()
                    .with_context(|| format!("failed to verify existence of {}", dest.display()))?,
                "{} exists",
                dest.display()
            );
        }

        Ok(())
    }

    #[cfg(not(windows))]
    #[test]
    fn test_make_relative_dest() {
        let pl = PlannedLink {
            src: PathBuf::from("/path/to/package/file"),
            dest: PathBuf::from("/path/to/target/file"),
            ty: LinkType::SymlinkRelative,
        };

        // the destination is supposed to be a path to the package file that is relative to the
        // destination. The full, unclean path for example:
        // /path/to/deeper/target/../../package/file
        let expected_src = PathBuf::from("../package/file");
        let actual_src = pl.get_src_relative_to_dest();

        assert_eq!(expected_src, actual_src);
    }


    #[cfg(windows)]
    #[test]
    fn test_make_relative_dest() {
        let pl = PlannedLink {
            src: PathBuf::from("C:\\path\\to\\package\\file"),
            dest: PathBuf::from("C:\\path\\to\\target\\file"),
            ty: LinkType::SymlinkRelative,
        };

        // the destination is supposed to be a path to the package file that is relative to the
        // destination. The full, unclean path for example:
        // /path/to/deeper/target/../../package/file
        let expected_src = PathBuf::from("..\\package\\file");
        let actual_src = pl.get_src_relative_to_dest();

        assert_eq!(expected_src, actual_src);
    }

    #[test]
    fn test_plan_empty_unboxing() -> anyhow::Result<()> {
        let package = tempfile::tempdir().context("failed to make test package")?;
        let package_path = package.path();
        let cli = BoxUnboxCli::new(package_path);
        let config = PackageConfig::new(package_path);

        let actual_result = UnboxPlan::plan_unboxing(config, &cli);

        assert!(
            actual_result.is_err(),
            "unexpectedly planned unboxing succesfully"
        );

        match actual_result.unwrap_err() {
            PlanningError::EmptyPlan => {}
            err => panic!("unboxing plan failed with unexpected error {err:?}"),
        }

        Ok(())
    }

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
            .collect::<UnboxPlan>();
        let actual_plan = UnboxPlan::plan_unboxing(config, &cli)?;

        #[cfg(debug_assertions)]
        assert_eq!(
            expected_plan.create_dirs, actual_plan.create_dirs,
            "unboxing plans disagree on create_dirs"
        );

        assert_eq!(
            expected_plan.efs, actual_plan.efs,
            "unboxing plan has unexpected file strategy"
        );

        assert_eq!(
            expected_plan.links.len(),
            actual_plan.links.len(),
            "unboxing plan has unexpected length"
        );

        for pl in &actual_plan.links {
            assert!(
                expected_plan.links.contains(pl),
                "unboxing plan contains unexpected planned link: {pl:?}"
            );
        }

        Ok(())
    }

    #[test]
    fn test_plan_unboxing_nested_config() -> anyhow::Result<()> {
        const TEST_NESTED_PACKAGE: &str = "folder1/";

        let package = make_tmp_tree().context("failed to make test package")?;
        let package_path = package.path();
        let cli = BoxUnboxCli::new(package_path);
        let config = PackageConfig::init(package_path, &cli)
            .context("failed to create test package config")?;

        // make nested config
        // NOTE: don't use TEST_TARGET here, we want to make sure the target change works
        #[cfg(not(windows))]
        let expected_nested_target = PathBuf::from("/some/other/test/target");
        #[cfg(windows)]
        let expected_nested_target = PathBuf::from("S:\\some\\other\\test\\target");
        let expected_nested_package = package_path.join(TEST_NESTED_PACKAGE);
        let mut nested_config = PackageConfig::new_with_target(
            expected_nested_package.clone(),
            expected_nested_target.clone(),
        );
        nested_config
            .exclude_pats
            .push(Regex::new("^test_ignore.*").expect("test Regex should compile"));
        nested_config
            .save_to_package()
            .context("failed to save nested test config to test package")?;

        let expected_target = PathBuf::from(TEST_TARGET);
        let expected_plan = TEST_PACKAGE_FILE_TAILS
            .into_iter()
            .filter(|tail| !tail.ends_with("test_ignore2.txt"))
            .map(|tail| {
                let dest = if let Some(file_tail) = tail.strip_prefix(TEST_NESTED_PACKAGE) {
                    expected_nested_target.join(file_tail)
                } else {
                    expected_target.join(tail)
                };

                PlannedLink {
                    src: package_path.join(tail),
                    dest,
                    ty: LinkType::SymlinkAbsolute,
                }
            })
            .collect::<UnboxPlan>();
        let actual_plan = UnboxPlan::plan_unboxing(config, &cli)?;

        #[cfg(debug_assertions)]
        assert_eq!(
            expected_plan.create_dirs, actual_plan.create_dirs,
            "unboxing plans disagree on create_dirs"
        );
        assert_eq!(
            expected_plan.efs, actual_plan.efs,
            "unboxing plan has unexpected file strategy"
        );

        assert_eq!(
            expected_plan.links.len(),
            actual_plan.links.len(),
            "unboxing plan has unexpected length"
        );

        for pl in &actual_plan.links {
            assert!(
                expected_plan.links.contains(pl),
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
    fn test_plan_unboxing_include_file() -> anyhow::Result<()> {
        let package = make_tmp_tree().context("failed to make test package")?;
        let package_path = package.path();
        let test_regex = Regex::new("^nested").expect("test regex should compile");
        let mut cli = BoxUnboxCli::new(package_path);
        cli.include_pats.push(test_regex.clone());
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
            .filter(|pl| {
                // only keep links whose source path components DO match the test regex
                pl.src
                    .components()
                    .any(|c| test_regex.is_match(&c.as_os_str().to_string_lossy()))
            })
            .collect::<UnboxPlan>();
        let actual_plan = UnboxPlan::plan_unboxing(config, &cli)?;

        #[cfg(debug_assertions)]
        assert_eq!(
            expected_plan.create_dirs, actual_plan.create_dirs,
            "unboxing plans disagree on create_dirs"
        );
        assert_eq!(
            expected_plan.efs, actual_plan.efs,
            "unboxing plan has unexpected file strategy"
        );

        assert_eq!(
            expected_plan.links.len(),
            actual_plan.links.len(),
            "unboxing plan has unexpected length"
        );

        for pl in &actual_plan.links {
            assert!(
                pl.src
                    .components()
                    .any(|c| test_regex.is_match(&c.as_os_str().to_string_lossy())),
                "unboxing plan contains excluded link {pl:?}"
            );
            assert!(
                expected_plan.links.contains(pl),
                "unboxing plan contains unexpected planned link: {pl:?}"
            );
        }

        Ok(())
    }

    #[test]
    fn test_plan_unboxing_include_overlaps_exclude() -> anyhow::Result<()> {
        let package = make_tmp_tree().context("failed to make test package")?;
        let package_path = package.path();
        let test_include_regex =
            Regex::new("^folder1$").expect("test include regex should compile");
        let test_exclude_regex = Regex::new("ignore").expect("test exclude regex should compile");
        let mut cli = BoxUnboxCli::new(package_path);
        cli.include_pats.push(test_include_regex.clone());
        cli.exclude_pats.push(test_exclude_regex.clone());
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
            .filter(|pl| {
                // only keep links whose source path components don't overlap
                let comps = pl
                    .src
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy())
                    .collect::<Vec<_>>();
                comps.iter().any(|c| test_include_regex.is_match(c))
                    && comps.iter().all(|c| !test_exclude_regex.is_match(c))
            })
            .collect::<UnboxPlan>();
        let actual_plan = UnboxPlan::plan_unboxing(config, &cli)?;

        #[cfg(debug_assertions)]
        assert_eq!(
            expected_plan.create_dirs, actual_plan.create_dirs,
            "unboxing plans disagree on create_dirs"
        );
        assert_eq!(
            expected_plan.efs, actual_plan.efs,
            "unboxing plan has unexpected file strategy"
        );

        assert_eq!(
            expected_plan.links.len(),
            actual_plan.links.len(),
            "unboxing plan has unexpected length"
        );

        for pl in &actual_plan.links {
            let comps = pl
                .src
                .components()
                .map(|c| c.as_os_str().to_string_lossy())
                .collect::<Vec<_>>();
            // asserts at least one component is included and NO components are excluded
            assert!(
                comps.iter().any(|c| test_include_regex.is_match(c))
                    && comps.iter().all(|c| !test_exclude_regex.is_match(c)),
                "unboxing plan contains excluded link {pl:?}"
            );
            assert!(
                expected_plan.links.contains(pl),
                "unboxing plan contains unexpected planned link: {pl:?}"
            );
        }

        Ok(())
    }

    #[test]
    fn test_plan_unboxing_include_dir() -> anyhow::Result<()> {
        let package = make_tmp_tree().context("failed to make test package")?;
        let package_path = package.path();
        let test_regex = Regex::new("^folder1$").expect("test regex should compile");
        let mut cli = BoxUnboxCli::new(package_path);
        cli.include_pats.push(test_regex.clone());
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
            .filter(|pl| {
                // only keep links whose source path components DO match the test regex
                pl.src
                    .components()
                    .any(|c| test_regex.is_match(&c.as_os_str().to_string_lossy()))
            })
            .collect::<UnboxPlan>();
        let actual_plan = UnboxPlan::plan_unboxing(config, &cli)?;

        #[cfg(debug_assertions)]
        assert_eq!(
            expected_plan.create_dirs, actual_plan.create_dirs,
            "unboxing plans disagree on create_dirs"
        );
        assert_eq!(
            expected_plan.efs, actual_plan.efs,
            "unboxing plan has unexpected file strategy"
        );

        assert_eq!(
            expected_plan.links.len(),
            actual_plan.links.len(),
            "unboxing plan has unexpected length"
        );

        for pl in &actual_plan.links {
            assert!(
                pl.src
                    .components()
                    .any(|c| test_regex.is_match(&c.as_os_str().to_string_lossy())),
                "unboxing plan contains excluded link {pl:?}"
            );
            assert!(
                expected_plan.links.contains(pl),
                "unboxing plan contains unexpected planned link: {pl:?}"
            );
        }

        Ok(())
    }

    #[test]
    fn test_plan_unboxing_exclude_dir() -> anyhow::Result<()> {
        let package = make_tmp_tree().context("failed to make test package")?;
        let package_path = package.path();
        let test_regex = Regex::new("^folder1$").expect("test regex should compile");
        let mut cli = BoxUnboxCli::new(package_path);
        cli.exclude_pats.push(test_regex.clone());
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
            .filter(|pl| {
                // only keep links whose source path components don't match the test regex
                !pl.src
                    .components()
                    .any(|c| test_regex.is_match(&c.as_os_str().to_string_lossy()))
            })
            .collect::<UnboxPlan>();
        let actual_plan = UnboxPlan::plan_unboxing(config, &cli)?;

        #[cfg(debug_assertions)]
        assert_eq!(
            expected_plan.create_dirs, actual_plan.create_dirs,
            "unboxing plans disagree on create_dirs"
        );
        assert_eq!(
            expected_plan.efs, actual_plan.efs,
            "unboxing plan has unexpected file strategy"
        );

        assert_eq!(
            expected_plan.links.len(),
            actual_plan.links.len(),
            "unboxing plan has unexpected length"
        );

        for pl in &actual_plan.links {
            assert!(
                !pl.src
                    .components()
                    .any(|c| test_regex.is_match(&c.as_os_str().to_string_lossy())),
                "unboxing plan contains excluded link {pl:?}"
            );
            assert!(
                expected_plan.links.contains(pl),
                "unboxing plan contains unexpected planned link: {pl:?}"
            );
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
        }]
        .into_iter()
        .collect::<UnboxPlan>();
        let actual_plan = UnboxPlan::plan_unboxing(config, &cli)?;

        #[cfg(debug_assertions)]
        assert_eq!(
            expected_plan.create_dirs, actual_plan.create_dirs,
            "unboxing plans disagree on create_dirs"
        );
        assert_eq!(
            expected_plan.efs, actual_plan.efs,
            "unboxing plan has unexpected file strategy"
        );

        assert_eq!(
            expected_plan.links.len(),
            actual_plan.links.len(),
            "unboxing plan has unexpected length"
        );

        for pl in &actual_plan.links {
            assert!(
                expected_plan.links.contains(pl),
                "unboxing plan contains unexpected planned link: {pl:?}"
            );
        }

        Ok(())
    }
}
