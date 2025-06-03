#![cfg(test)]

use std::path::PathBuf;

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

pub const TEST_TARGET: &str = "/path/to/test/target";

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
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create test dir '{parent:?}'"))?;
        // use file path as file contents
        std::fs::write(&full_path, full_path.clone().to_string_lossy().as_bytes())
            .with_context(|| format!("failed to create test file '{full_path:?}'"))?;
    }

    // create demo config with home dir as target
    let conf = PackageConfig::new(root);
    conf.save_to_package()
        .context("failed to save test config")?;

    Ok(temp_dir)
}
