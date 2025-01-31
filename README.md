# boxunbox

`boxunbox` is a simple symlinker inspired by [GNU `stow`](https://www.gnu.org/software/stow/).

## Why?

`stow` is quite popular and obviously works well for so many people, so why reinvent the wheel?

To be honest, I love `stow`, but there are a couple bones I have to pick with it. First, the `.stowrc` files. If I stow a package named `home`, for example, it does not use the config at `home/.stowrc`, it uses `./.stowrc` or a global config, if you have one. This isn't very intuitive in my opinion, so `boxunbox`, by comparison, places its config files (`.unboxrc.ron`) _inside_ package directories. It also allows creating `.unboxrc.ron` files in _any_ sub-directory of your package and that will be treated as the config for that directory and its contents. Moving on, I also don't like that `stow` symlinks directories by default. I wrote `boxunbox` to **only symlink files** because I had issues where a directory would get linked, new files would then get created in the linked directory, thus polluting the original location which is quite annoying when you use it to manage dotfiles. Maybe in the future I'll implement `stow`'s least link algorithm, but not today. Lastly, `stow` only creates relative symlinks. By default, `boxunbox` creates absolute symlinks.

## TODO

- [-] Flag for relative symlinks
