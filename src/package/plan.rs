use std::{
    fs, iter,
    path::{Path, PathBuf},
    sync::LazyLock,
};

use anyhow::Context;
use regex::Regex;

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
    /// it checks that `root_package` exists. Then, it iterates over the entire contents of the
    /// directory and batch checks file permissions. It then iterates once more, this time
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
    /// - The link target for a file already exists.
    pub fn try_from_package<P: AsRef<Path>>(
        root_package: P,
        root_config: PackageConfig,
    ) -> Result<Self, UnboxError> {
        static RC_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new("\\.unboxrc").unwrap());

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
        let pkg_entry_paths = walkdir::WalkDir::new(&root_package)
            .sort_by_file_name()
            .into_iter()
            .skip(1) // skip root package
            .map(|res| res.map(|ent| ent.path().to_path_buf()))
            .collect::<Result<Vec<PathBuf>, _>>()?;

        // plan your moves first before doing anything in case something fails; don't want to get
        // halfway done unboxing just to realize you have to box it all back up!
        // TODO: rollback plans on error (consider a plan struct)
        let mut planned_links = Vec::new();
        let mut planned_dirs = Vec::new();

        for path in pkg_entry_paths {
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
                .chain(iter::once(&*RC_REGEX))
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

            // if the target exists, is a directory, and `path_is_dir`, just skip it; otherwise,
            // return an error.
            if new_target
                .try_exists()
                .with_context(|| format!("failed to verify existence of {new_target:?}"))?
            {
                // if both the original and target are already directories
                if path_is_dir && new_target.is_dir() {
                    continue;
                }

                // exists, but is file/symlink
                return Err(UnboxError::TargetAlreadyExists {
                    package_entry: path.clone(),
                    target_entry: new_target.clone(),
                });
            }

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
                fs::create_dir(dir).with_context(|| format!("failed to mkdir {dir:?}"))
            })?;
            links.iter().try_for_each(|(src, dest)| {
                os_symlink(src, dest)
                    .with_context(|| format!("failed to symlink {src:?} -> {dest:?}"))
            })?;
        }

        Ok(())
    }
}
