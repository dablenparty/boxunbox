use std::{fs::DirEntry, path::Path};

/// Boxes a package entry up from `target`. The `pkg_entry`'s file name is used to make the symlink
/// path.
///
/// # Arguments
///
/// - `pkg_entry` - The [`DirEntry`] to box.
/// - `target` - The `&Path` to box things up from.
///
/// # Errors
///
/// An error is returned if one occurs removing the symlink.
#[inline]
fn box_package_entry(pkg_entry: &DirEntry, target: &Path) -> anyhow::Result<()> {
    let link_path = target.join(pkg_entry.file_name());

    std::fs::remove_file(link_path)?;

    Ok(())
}

fn main() -> anyhow::Result<()> {
    todo!("implement box binary")
}
