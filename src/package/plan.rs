use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use anyhow::Context;

use crate::{
    package::{PackageConfig, errors::ParseError},
    utils::os_symlink,
};

use super::errors::UnboxError;

#[derive(Debug)]
pub struct UnboxPlan {
    links: Vec<(PathBuf, PathBuf)>,
    dirs: Vec<PathBuf>,
    config: PackageConfig,
}

impl TryFrom<PackageConfig> for UnboxPlan {
    type Error = UnboxError;

    #[allow(clippy::too_many_lines)]
    fn try_from(root_config: PackageConfig) -> Result<Self, Self::Error> {
        let root_package = root_config.package.clone();

        match root_package.try_exists() {
            Ok(true) => {}
            Ok(false) | Err(_) => return Err(UnboxError::PackageNotFound(root_package.clone())),
        }

        if root_config.link_root {
            return Ok(Self {
                links: vec![(root_config.package.clone(), root_config.target.clone())],
                dirs: Vec::new(),
                config: root_config,
            });
        }

        // TODO: least links algorithm?
        // At the very least, I want a way to choose between the algorithms

        /*
        NOTE: currently only creates file symlinks, not directories
        I chose this because I had issues where the directory would get linked, then files
        placed there by other programs would show up in the original location, which I don't
        want.
        */

        let mut config_stack: Vec<PackageConfig> = vec![root_config.clone()];

        /// Utility macro that expands to get the last config off the config stack.
        macro_rules! clone_last_config {
            () => {
                config_stack
                    .last()
                    .expect("there should be at least one config in the stack")
                    .clone()
            };
        }

        // WARN: don't use any intermediate methods on the iterator. The iterator is modified later
        // by a call to `skip_current_dir()` which is a method on the WalkDir iterator only.
        let mut pkg_entry_path_iter = walkdir::WalkDir::new(&root_package)
            .sort_by_file_name()
            .into_iter();

        // plan your moves first before doing anything in case something fails; don't want to get
        // halfway done unboxing just to realize you have to box it all back up!
        let mut planned_links = Vec::new();
        let mut planned_dirs = Vec::new();

        // skip root dir, it's handled separately
        let _ = pkg_entry_path_iter.next();

        // manual loop condition allows in-loop modification of iterator which is how I achieved
        // ignoring dirs
        while let Some(res) = pkg_entry_path_iter.next() {
            // essentially guards against errors; if even ONE occurs, abort and return it.
            let path = res?.into_path();
            let path_is_dir = path.is_dir();

            let last_config = clone_last_config!();

            // This basically says: if we're in a subdir of the last config, keep going;
            // otherwise, pop the config off the stack, but panic if the stack was
            // empty.
            assert!(
                path.starts_with(&last_config.package) || config_stack.pop().is_some(),
                "there should be at least one config on the stack"
            );

            // If this path is a directory, read its config and push it to the stack.
            // I went back and forth on whether or no to do this, but I ended up finding use cases
            // where I needed this feature and, well, _none_ where I didn't. So here it is.
            if path_is_dir {
                // if the config exists, add it to the stack; if not, don't care
                // any other error, ERROR!!
                match PackageConfig::try_from_package(&path) {
                    Ok(config) => config_stack.push(config),
                    Err(ParseError::ConfigNotFound(_)) => {}
                    Err(err) => return Err(err.into()),
                }
            }

            let last_config = clone_last_config!();

            let file_name = path
                .file_name()
                .unwrap_or_else(|| path.as_os_str())
                .to_string_lossy();

            // check all ignore patterns in the stack
            if config_stack
                .iter()
                .flat_map(|conf| conf.ignore_pats.as_slice())
                .any(|re| re.is_match(&file_name))
            {
                if path_is_dir {
                    // ignores this dir and all children
                    pkg_entry_path_iter.skip_current_dir();
                }

                #[cfg(debug_assertions)]
                println!("ignoring {path:?} (ignore pattern)");

                continue;
            }

            let PackageConfig {
                package,
                target,
                use_relative_links,
                no_create_dirs,
                ..
            } = last_config;

            // /path/to/package/entry -> /entry
            let stripped = path.strip_prefix(&package).unwrap_or_else(|err| {
                unreachable!(
                    "failed to strip package prefix '{package:?}' from package entry '{path:?}': {err:?}"
                )
            });
            // /entry -> /path/to/target/entry
            let new_target = target.join(stripped);

            let new_target = if use_relative_links {
                let target_parent = new_target.parent().unwrap_or(&new_target);
                pathdiff::diff_paths(&path, target_parent).ok_or(UnboxError::PathDiffError {
                    path: path.clone(),
                    base: target_parent.to_path_buf(),
                })?
            } else {
                new_target
            };

            if path_is_dir {
                if !no_create_dirs {
                    planned_dirs.push(new_target);
                }
            } else {
                planned_links.push((path, new_target));
            }
        } // while let Some(res) = pkg_entry_path_iter.next()

        let plan = Self {
            dirs: planned_dirs,
            links: planned_links,
            config: root_config,
        };
        Ok(plan)
    }
}

impl UnboxPlan {
    /// Generate an [`UnboxPlan`] from a given `root_package`. This is quite involved. First,
    /// it checks that `root_package` exists. It then iterates over the directory contents,
    /// planning which directories to create and which files to symlink based on the ignore
    /// patterns.
    ///
    /// # Arguments
    ///
    /// - `root_package` - Root package to plan the unboxing from.
    ///
    /// # Errors
    ///
    /// An error is returned if:
    ///
    /// - `root_package` is not found/readable
    /// - Any [`PackageConfig`]'s fail to parse.
    /// - Any package sub-dirs or files cannot be read.
    pub fn new(root_config: PackageConfig) -> Result<Self, UnboxError> {
        Self::try_from(root_config)
    }

    /// Check if this [`UnboxPlan`] can be executed. Note that this does not guarantee that an
    /// error will not happen, but can at least check as much as possible. This function is not
    /// called when running with the `--box` flag.
    ///
    /// # Errors
    ///
    /// An error is retured if a target link already exists or if a directory already exists and
    /// the running user doesn't have read/write permissions.
    pub fn check_plan(&self) -> Result<(), UnboxError> {
        #[cfg(unix)]
        fn file_is_writeable<P: AsRef<Path>>(file: P) -> bool {
            use std::os::unix::fs::MetadataExt;

            // read/write access, per docs:
            // https://doc.rust-lang.org/std/os/unix/fs/trait.MetadataExt.html#tymethod.mode
            file.as_ref()
                .metadata()
                .is_ok_and(|meta| (meta.mode() & 0o600) != 0)
        }

        let Self {
            links,
            dirs,
            config,
        } = self;

        // if the target dir already exists, but is supposed to be a symlink, ERROR!!
        if config.link_root && config.target.is_dir() {
            return Err(UnboxError::TargetAlreadyExists {
                package_entry: config.package.clone(),
                target_entry: config.target.clone(),
            });
        }

        // verify dirs as you go along the files to avoid having to iterate self.dirs
        let mut verified_dirs = HashSet::with_capacity(dirs.capacity());

        for (_, dest) in links {
            // check if the running user can write to the parent directory
            let parent = dest
                .parent()
                .map_or_else(|| PathBuf::from("/"), Path::to_path_buf);

            // if the dir is already verified or doesn't exist, just continue
            if verified_dirs.contains(&parent)
                || !parent
                    .try_exists()
                    .with_context(|| format!("failed to verify existence of {parent:?}"))?
            {
                continue;
            }

            // otherwise, the dir exists, so check if it's writeable
            if file_is_writeable(&parent) {
                verified_dirs.insert(parent);
            } else {
                return Err(UnboxError::NoWritePermissions(parent.clone()));
            }
        }

        Ok(())
    }

    /// Execute this [`UnboxPlan`]. You may want to call [`UnboxPlan::check_plan`] before this.
    ///
    /// # Errors
    ///
    /// An error is returned if the directories or symlinks cannot be created.
    pub fn execute(&self) -> anyhow::Result<()> {
        let Self {
            links,
            dirs,
            config,
        } = self;

        println!("ready to unbox: {self:#?}");

        if config.dry_run {
            // TODO: better dry run output (colors?)
            println!("dry run, not executing");
        } else {
            if !config.link_root {
                fs::create_dir_all(&config.target)
                    .with_context(|| format!("failed to create target {:?}", config.target))?;
            }

            // make directories first, then link target files
            dirs.iter().try_for_each(|dir| {
                // use create_dir because they should be in hierarchical order
                if dir
                    .try_exists()
                    .with_context(|| format!("failed to verify existence of dir {dir:?}"))?
                {
                    Ok(())
                } else {
                    fs::create_dir(dir).with_context(|| format!("failed to mkdir {dir:?}"))
                }
            })?;

            links.iter().try_for_each(|(src, dest)| {
                if dest
                    .try_exists()
                    .with_context(|| format!("failed to verify existence of {dest:?}"))?
                {
                    // If new_target exists, don't plan it; however, only return an error if they're
                    // not ignored.
                    if config.force {
                        fs::remove_file(dest)
                            .with_context(|| format!("failed to force remove {dest:?}"))?;
                    } else {
                        return if config.ignore_exists {
                            Ok(())
                        } else {
                            Err(UnboxError::TargetAlreadyExists {
                                package_entry: src.clone(),
                                target_entry: dest.clone(),
                            })
                        };
                    }
                }

                os_symlink(src, dest)
                    .with_context(|| format!("failed to symlink {src:?} -> {dest:?}"))
                    .map_err(UnboxError::from)
            })?;
        }

        Ok(())
    }

    /// Rollback this [`UnboxPlan`] by iterating over the planned links and removing their
    /// destinations, if they exist and are symlinks.
    ///
    /// # Errors
    ///
    /// An error may occur while checking existence, reading metadata, or removing the symlink.
    pub fn rollback(&self) -> anyhow::Result<()> {
        if self.config.dry_run {
            println!("ready to box: {self:#?}");
            return Ok(());
        }

        self.links.iter().try_for_each(|(_, dest)| {
            // existence check is implied by symlink_metadata
            if dest
                .try_exists()
                .with_context(|| format!("failed to check existence of {dest:?}"))?
                && dest
                    .symlink_metadata()
                    .with_context(|| format!("failed to read metadata of {dest:?}"))?
                    .is_symlink()
            {
                fs::remove_file(dest)
                    .with_context(|| format!("failed to remove symlink {dest:?}"))?;
            }

            Ok(())
        })
    }
}
