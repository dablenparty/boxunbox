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
pub fn make_tmp_tree() {
    todo!("make_tmp_tree")
}
