#![cfg(test)]

use std::fs;

use anyhow::Context;
use tempfile::TempDir;

use crate::package::PackageConfig;

pub const TEST_PACKAGE_FILE_TAILS: [&str; 6] = [
    "folder1/nested1.txt",
    "folder1/test_ignore2.txt",
    "folder2/nested2.txt",
    "folder2/nested2 again.txt",
    "test.txt",
    "test_ignore.txt",
];

#[cfg(not(windows))]
pub const TEST_TARGET: &str = "/path/to/test/target";

#[cfg(windows)]
pub const TEST_TARGET: &str = "T:\\path\\to\\test\\target";

/// Compare two [`Vec`]s by converting their values to [`String`]s and comparing those. Returns
/// `true` if `left` and `right` are the same length, same order, and all elements are equal. This
/// is most useful for comparing [`Vec`]s of types that do not implement [`PartialEq`], but _do_
/// implement [`ToString`], such as [`regex::Regex`].
///
/// # Arguments
///
/// - `left` - Left [`Vec`]
/// - `right` - Right [`Vec`]
pub fn vec_string_compare<S: ToString>(left: &[S], right: &[S]) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .map(ToString::to_string)
            .zip(right.iter().map(ToString::to_string))
            .all(|(l, r)| l == r)
}

/// Creates a new temporary file tree for use in integration tests. Each call to this function will
/// create a _new_ temporary directory; however, every directory will have the same structure:
///
/// ```text
/// <tempdir>
/// ├── folder1
/// │   ├── nested1.txt
/// │   └── test_ignore2.txt
/// ├── folder2
/// │   ├── nested2.txt
/// │   └── 'nested2 again.txt'
/// ├── test.txt
/// └── test_ignore.txt
/// ```
pub fn make_tmp_tree() -> anyhow::Result<TempDir> {
    let temp_dir = tempfile::tempdir().context("failed to create tempdir")?;
    let root = temp_dir.path();
    for file in &TEST_PACKAGE_FILE_TAILS {
        let full_path = root.join(file);
        let parent = full_path.parent().unwrap();
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create test dir '{parent:?}'"))?;
        // use file path as file contents
        fs::write(&full_path, full_path.clone().to_string_lossy().as_bytes())
            .with_context(|| format!("failed to create test file '{full_path:?}'"))?;
    }

    // create demo config with home dir as target
    let conf = PackageConfig::new(root);
    conf.save_to_package()
        .context("failed to save test config")?;

    Ok(temp_dir)
}
