use std::sync::LazyLock;

/// Lazy wrapper around [`directories_next::BaseDirs::new`].
pub static BASE_DIRS: LazyLock<directories_next::BaseDirs> = LazyLock::new(|| {
    directories_next::BaseDirs::new().expect("failed to locate user home directory")
});
