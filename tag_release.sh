#!/usr/bin/env bash

set -eo pipefail

# HOW TO USE
# Simply change the version string in Cargo.toml, but DO NOT COMMIT THE CHANGE!
# Just run this script in the dirty repo, it'll commit everything for you.

if [[ "$OSTYPE" =~ ^darwin.* ]]; then
  # alias macOS sed to GNU sed for simplicity
  sed() {
    gsed "$@"
  }
fi

pkgver="${ rg --color=never -Noe '^version\s*=\s*"(.+?)"$' --replace '$1' boxunbox/Cargo.toml; }"
gittag="v$pkgver"
if [[ -n "${ git tag --list "$gittag"; }" ]]; then
  echo "warn: tag $gittag already exists, it will be removed" 1>&2
  git tag -d "$gittag"
fi

# commit version bump
cargo update
git add Cargo.lock boxunbox/Cargo.toml
git commit -m "chore: bump version ($gittag)"

# tag new commit for git cliff
git tag -- "$gittag" "${ git rev-parse HEAD; }"

# update the CHANGELOG and amend it to the tagged commit
git cliff -o CHANGELOG.md
git add CHANGELOG.md
git commit --amend --no-edit --allow-empty
# re-tag commit after amending CHANGELOG because hash changes
git tag --force -- "$gittag" "${ git rev-parse HEAD; }"
