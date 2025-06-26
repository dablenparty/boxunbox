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

## Installation

At the time of writing, there are currently two supported installation methods. I used to publish to `crates.io` as well, but after redesigning the codebase to use subcrates, publishing to `crates.io` has become too convoluted. It requires that I publish all subcrates in order to package the binary, but I cannot publish the subcrates without packaging, making the process circular. I could probably get around this by restructuring my project yet again, but it's not worth it to me.

### Arch User Repository (AUR)

There are two AUR packages:

- [boxunbox](https://aur.archlinux.org/packages/boxunbox): latest tagged release
- [boxunbox-git](https://aur.archlinux.org/packages/boxunbox-git): latest commit on `main`

They aren't usually different by much. I recommend installing with an AUR helper:

```bash
paru -S boxunbox
# or
paru -S boxunbox-git
```

### Manual Local Installation

First, clone the repostiory, then compile/install with one command:

```bash
git clone https://github.com/dablenparty/boxunbox.git
cargo install --path boxunbox
```

Explore `cargo install --help` for more installation options.

## Configuration

For the CLI, read the output of `unbox --help`. See the [example config](example.bub.toml) for an overview of the config file. Alternatively, you can just view the [package config struct definition](src/package.rs) if you're comfortable with Rust.

Package config files are stored as `.bub.toml` or `.bub.<platform>.toml` for OS-specific configs. See [this doc page](https://doc.rust-lang.org/std/env/consts/constant.OS.html) for a list of possible values for `<platform>`, although the CLI has a flag `-o` that can create one for you automatically. OS-specific configs are always preferred when they exist.
