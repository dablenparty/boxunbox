use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use anyhow::Context;

use crate::{
    package::{errors::ParseError, PackageConfig},
    utils::os_symlink,
};

use super::errors::UnboxError;

#[derive(Debug)]
pub struct UnboxPlan {
    links: Vec<(PathBuf, PathBuf)>,
    dirs: Vec<PathBuf>,
    config: PackageConfig,
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
    pub fn try_from_package<P: AsRef<Path>>(
        root_package: P,
        root_config: PackageConfig,
    ) -> Result<Self, UnboxError> {
        let root_package = root_package.as_ref().to_path_buf();

        match root_package.try_exists() {
            Ok(true) => {}
            Ok(false) | Err(_) => return Err(UnboxError::PackageNotFound(root_package.clone())),
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

        // essentially guards against errors; if even ONE occurs, abort and return it.
        let pkg_entry_path_iter = walkdir::WalkDir::new(&root_package)
            .sort_by_file_name()
            .into_iter()
            .skip(1) // skip root package
            .map(|res| res.map(|ent| ent.path().to_path_buf()));

        // plan your moves first before doing anything in case something fails; don't want to get
        // halfway done unboxing just to realize you have to box it all back up!
        let mut planned_links = Vec::new();
        let mut planned_dirs = Vec::new();

        for path in pkg_entry_path_iter {
            let path = path?;
            let path_is_dir = path.is_dir();

            let last_config = clone_last_config!();

            // If we're in a subdir of the last config, keep using it. Otherwise, pop it off the
            // stack and get the next one.
            let last_config = if path.starts_with(&last_config.package) {
                last_config
            } else {
                let _ = config_stack
                    .pop()
                    .expect("there should be at least one config in the stack");
                clone_last_config!()
            };

            // read the config of this subdir
            // if the config exists, add it to the stack; if not, don't care
            match PackageConfig::try_from_package(&path) {
                Ok(config) => config_stack.push(config),
                Err(ParseError::FileNotFound(_)) => {}
                Err(err) => return Err(err.into()),
            }

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
                #[cfg(debug_assertions)]
                println!("ignoring file {path:?}");

                continue;
            }

            let PackageConfig {
                package, target, ..
            } = last_config;

            // /path/to/package/entry -> /entry
            let stripped = path.strip_prefix(&package).unwrap_or_else(|err| {
                unreachable!(
                    "failed to strip package prefix '{package:?}' from package entry '{path:?}': {err:?}"
                )
            });
            // /entry -> /path/to/target/entry
            let new_target = target.join(stripped);

            if path_is_dir {
                planned_dirs.push(new_target);
            } else {
                planned_links.push((path, new_target));
            }
        }

        let plan = Self {
            dirs: planned_dirs,
            links: planned_links,
            config: root_config,
        };
        Ok(plan)
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

        // verify dirs as you go along the files to avoid having to iterate self.dirs
        let mut verified_dirs = HashSet::with_capacity(self.dirs.capacity());

        for (src, dest) in &self.links {
            if dest
                .try_exists()
                .with_context(|| format!("failed to verify existence of {dest:?}"))?
            {
                return Err(UnboxError::TargetAlreadyExists {
                    package_entry: src.clone(),
                    target_entry: dest.clone(),
                });
            }

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
            // make directories first, then link target files
            dirs.iter().try_for_each(|dir| {
                // use create_dir because they should be in hierarchal order
                dir.try_exists()
                    .with_context(|| format!("failed to verify existence of dir {dir:?}"))
                    .and_then(|exists| {
                        if exists {
                            Ok(())
                        } else {
                            fs::create_dir(dir).with_context(|| format!("failed to mkdir {dir:?}"))
                        }
                    })
            })?;
            links.iter().try_for_each(|(src, dest)| {
                os_symlink(src, dest)
                    .with_context(|| format!("failed to symlink {src:?} -> {dest:?}"))
            })?;
        }

        Ok(())
    }

    pub fn rollback(&self) -> anyhow::Result<()> {
        self.links.iter().try_for_each(|(_, dest)| {
            if dest
                .try_exists()
                .with_context(|| format!("failed to check existence of {dest:?}"))?
            {
                fs::remove_file(dest).with_context(|| format!("failed to remove file {dest:?}"))?;
            }

            Ok(())
        })
    }
}
