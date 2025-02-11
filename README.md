# boxunbox

`boxunbox` is a simple symlinker inspired by [GNU `stow`](https://www.gnu.org/software/stow/).

## Why?

`stow` is quite popular and obviously works well for so many people, so why reinvent the wheel?

To be honest, I love `stow`, but I have a couple of bones to pick with it. First, the `.stowrc` files; if I stow a package named `home`, for example, it does not use the config at `home/.stowrc`, it uses whatever is found in the current directory (`./.stowrc`) or a global config, if you have one. This isn't very intuitive in my opinion; therefore, `boxunbox` places its config files (`.unboxrc.ron`) _inside_ the package directories it is unboxing (i.e. "`stow`-ing"). It also allows creating `.unboxrc.ron` files in _any_ sub-directory of your package and having that be treated as the config for that directory and its contents.

Moving on, I also don't like that `stow` symlinks directories by default. I wrote `boxunbox` to **only symlink files** by default because I had issues where a directory would get linked and then new files would get created in the linked directory, thus polluting the original location. ~Maybe~ In the future, I'll implement `stow`'s least link algorithm, but not today.

Lastly, `stow` only creates relative symlinks. By default, `boxunbox` creates absolute symlinks with an option for relative links.
