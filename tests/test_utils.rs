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

    todo!("create file structure and populate text files");

    Ok(temp_dir)
}
