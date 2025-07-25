#!/usr/bin/env bash

set -eo pipefail

# HOW TO USE
# Once you've pushed a tagged release to GitHub, use this script to update the tagged PKGBUILD based on the new release.

pkgver="$(rg --color=never -Noe '^version\s*=\s*"(.+?)"$' --replace '$1' boxunbox/Cargo.toml)"

cd aur/boxunbox
# replace the version
sed -Ei "s/^pkgver=.+?/pkgver=$pkgver/" PKGBUILD
# generate checksums for the new version
checksums="$(makepkg --nocolor --geninteg -p PKGBUILD | rg --color=never -o 'sha256sums=(.+)')"
sed -Ei "s/^sha256sums=.+?/$checksums/" PKGBUILD
makepkg --printsrcinfo >.SRCINFO

git add .
git commit -m "build: v$pkgver"
cd ../..
# commit submodule change
git add aur/*
git commit -m "chore: PKGBUILD v$pkgver"
