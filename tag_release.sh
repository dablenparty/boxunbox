#!/usr/bin/env bash

set -e

# HOW TO USE
# Simply change the version string in Cargo.toml, but DO NOT COMMIT THE CHANGE!
# Just run this script in the dirty repo, it'll commit everything for you.

pkgver="$(sed -En 's/^version\s*=\s*"(.+?)"$/\1/p' Cargo.toml)"
if [[ -n "$(git tag --list "v$pkgver")" ]]; then
  echo "error: tag v$pkgver already exists. Did you forget to update the version in Cargo.toml?" 1>&2
  exit 1
fi

# update Cargo.lock
cargo update

# commit version bump
git add Cargo.*
git commit -m "chore: bump version (v$pkgver)"

# tag new commit
git tag -- "v$pkgver" "$(git rev-parse HEAD)"

# update the CHANGELOG and amend it to the tagged commit
git cliff -o CHANGELOG.md
git add CHANGELOG.md
git commit --amend --no-edit --allow-empty
# re-tag commit after amend
git tag --force -- "v$pkgver" "$(git rev-parse HEAD)"

cd aur/boxunbox || exit 1
# update PKGBUILD
sed -Ei "s/^pkgver=.+?/pkgver=$pkgver/" PKGBUILD
makepkg --printsrcinfo >.SRCINFO
git add .
git commit -m "build: v$pkgver"
cd ../.. || exit 1
