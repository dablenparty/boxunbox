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

pkgver="$(rg --color=never -Noe '^version\s*=\s*"(.+?)"$' --replace '$1' boxunbox/Cargo.toml)"
if [[ -n "$(git tag --list "v$pkgver")" ]]; then
  echo "warn: tag v$pkgver already exists, it will be removed" 1>&2
  git tag -d "v$pkgver"
fi

# commit version bump
cargo update
git add Cargo.lock boxunbox/Cargo.toml
git commit -m "chore: bump version (v$pkgver)"

# tag new commit for git cliff
git tag -- "v$pkgver" "$(git rev-parse HEAD)"

# update the CHANGELOG and amend it to the tagged commit
git cliff -o CHANGELOG.md
git add CHANGELOG.md
git commit --amend --no-edit --allow-empty
# re-tag commit after amending CHANGELOG because hash changes
git tag --force -- "v$pkgver" "$(git rev-parse HEAD)"
