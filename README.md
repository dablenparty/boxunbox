# boxunbox

`boxunbox` is a simple symlinker inspired by [GNU `stow`](https://www.gnu.org/software/stow/).

## Comparison to GNU `stow`

In all honesty, I wanted to be able to control where _each individual folder_ got linked and the way `stow` handles the `.stowrc` files isn't very intuitive in my opinion. Therefore, `boxunbox` stores its config files in the packages themselves. For example, `unbox zsh` would use the config _inside_ the `zsh/` folder, not the current working directory like `stow`. This means you can also [nest config files](demo/src/folder2/.unboxrc.ron) and they'll be respected by design.

| `boxunbox`                                                            | `stow`                                   |
| --------------------------------------------------------------------- | ---------------------------------------- |
| Absolute (default) and relative links                                 | Relative links only                      |
| Per-package configs                                                   | Per-operation config                     |
| OS-specific configs                                                   | N/A                                      |
| Only symlinks files by default, creating directories as needed        | Creates as few symlinks as possible      |
| Re-creates directory structure in target until nested config is found | Re-creates directory structure in target |

## Configuration

For CLI args, read the output of `unbox --help`. See the [example config](example.unboxrc.ron) for an overview of the config file. Alternatively, you can just view the [package config struct definition](src/package.rs) if you're comfortable with Rust.

Package config files are stored as `.unboxrc.ron` or `.unboxrc.<platform>.ron` for OS-specific configs. See [this doc page](https://doc.rust-lang.org/std/env/consts/constant.OS.html) for a list of possible values for `<platform>`, although the CLI has a flag that can create one for you automatically. OS-specific configs are always preferred when they exist.
