use std::os::unix::ffi::OsStrExt;

use anyhow::Context;
use tempfile::TempDir;

/// Creates a new temporary file tree for use in integration tests. Each call to this function will
/// create a _new_ temporary directory; however, every directory will have the same structure:
///
/// ```text
/// <tempdir>
/// └── src
///     ├── folder1
///     │   ├── nested1.txt
///     │   └── test_ignore2.txt
///     ├── folder2
///     │   ├── nested2.txt
///     │   └── 'nested2 again.txt'
///     ├── test.txt
///     └── test_ignore.txt
/// ```
pub fn make_tmp_tree() -> anyhow::Result<TempDir> {
    const FILES_TO_CREATE: [&str; 6] = [
        "src/folder1/nested1.txt",
        "src/folder1/test_ignore2.txt",
        "src/folder2/nested2.txt",
        "src/folder2/nested2 again.txt",
        "src/test.txt",
        "src/test_ignore.txt",
    ];
    let temp_dir = tempfile::tempdir().context("failed to create tempdir")?;
    let root = temp_dir.path();
    for file in &FILES_TO_CREATE {
        let full_path = root.join(file);
        let parent = full_path.parent().unwrap();
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create test dir '{parent:?}'"))?;
        // use file path as file contents
        std::fs::write(&full_path, full_path.clone().into_os_string().as_bytes())
            .with_context(|| format!("failed to create test file '{full_path:?}'"))?;
    }

    Ok(temp_dir)
}
