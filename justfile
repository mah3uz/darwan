set shell := ["bash", "-euo", "pipefail", "-c"]

root := justfile_directory()
bin := root / "target/release"
ver := `sed -n 's/^pkgver=//p' packaging/aur/darwan/PKGBUILD`

# List the recipes
default:
    @just --list --unsorted

# Build every binary
build:
    cargo build --release --workspace

# Rust, QML and theme-lint tests
test:
    cargo test --release --workspace

# Formatting and clippy
lint:
    cargo fmt --all --check
    cargo clippy --release --workspace --all-targets -- -D warnings

# Load every theme offscreen and type the password (slow); e.g. `just check --no-fonts`
check *args: build
    DARWAN_DATA_DIR={{root}} {{bin}}/darwan check --all {{args}}

# The CLI against this checkout, e.g. `just darwan list`; no arguments opens the TUI
darwan *args: build
    DARWAN_DATA_DIR={{root}} {{bin}}/darwan {{args}}

# The GUI against this checkout
gui: build
    DARWAN_DATA_DIR={{root}} {{bin}}/darwan-gui

# Full-screen preview of a theme, e.g. `just preview genshin --at 18:30`
preview theme *args: build
    DARWAN_DATA_DIR={{root}} {{bin}}/darwan preview {{theme}} {{args}}

# Copy runtime/theme-kit into the named themes, or refresh every theme that has it, e.g. `just theme-kit nothing`
theme-kit *ids:
    cd {{root}} && for id in {{ if ids == "" { "$(find themes -path '*/darwan/Custom.qml' | sed 's|^themes/||; s|/darwan/Custom.qml$||')" } else { ids } }}; do mkdir -p "themes/$id/darwan" && cp runtime/theme-kit/darwan/*.qml "themes/$id/darwan/" && echo "themes/$id/darwan"; done

# Build the package from the committed HEAD into dist/, as the AUR build does from a tag
package:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -n "$(git status --porcelain)" ]; then
      echo "note: packaging HEAD; uncommitted changes are left out" >&2
    fi
    rm -rf dist/build
    mkdir -p dist/build
    git archive --prefix=darwan-{{ver}}/ -o dist/build/darwan-{{ver}}.tar.gz HEAD
    cp packaging/aur/darwan/PKGBUILD dist/build/
    (cd dist/build && makepkg -fs --noconfirm --skipchecksums)
    mv dist/build/darwan-{{ver}}-*-x86_64.pkg.tar.zst dist/
    rm -rf dist/build
    ls dist/darwan-{{ver}}-*-x86_64.pkg.tar.zst

# Install the package built by `just package`
install: package
    sudo pacman -U dist/darwan-{{ver}}-*-x86_64.pkg.tar.zst

# Remove the installed package
uninstall:
    sudo pacman -R darwan

# After pushing tag v<pkgver>: AUR checksums, .SRCINFO and the darwan-bin release asset
release:
    packaging/release.sh

# Publish packaging/aur to the AUR, through throwaway clones in dist/aur
aur:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -n "$(git status --porcelain packaging/aur)" ]; then
      echo "commit packaging/aur first" >&2
      exit 1
    fi
    url=$(sed -n "s/^url='\(.*\)'/\1/p" packaging/aur/darwan/PKGBUILD)
    if ! curl -fsIL -o /dev/null "$url/releases/download/v{{ver}}/darwan-{{ver}}-x86_64.pkg.tar.zst"; then
      echo "the v{{ver}} GitHub Release has no package yet, which darwan-bin downloads" >&2
      exit 1
    fi
    for pkg in darwan darwan-bin; do
      src=packaging/aur/$pkg
      if grep -q "^sha256sums=('SKIP')" "$src/PKGBUILD"; then
        echo "$pkg: no checksum; run just release first" >&2
        exit 1
      fi
      if ! diff -q <(cd "$src" && makepkg --printsrcinfo) "$src/.SRCINFO" >/dev/null; then
        echo "$pkg: .SRCINFO is stale; run just srcinfo and commit" >&2
        exit 1
      fi
      dir=dist/aur/$pkg
      rm -rf "$dir"
      git clone -q "ssh://aur@aur.archlinux.org/$pkg.git" "$dir" 2>/dev/null
      cp "$src"/{PKGBUILD,.SRCINFO,LICENSE} "$dir"/
      git -C "$dir" add PKGBUILD .SRCINFO LICENSE
      if git -C "$dir" diff --cached --quiet; then
        echo "$pkg: already up to date"
      else
        rel=$(sed -n 's/^pkgrel=//p' "$src/PKGBUILD")
        git -C "$dir" commit -q -m "Update to {{ver}}-$rel"
        # The AUR only accepts master, whatever init.defaultBranch named the clone's branch.
        git -C "$dir" push -q origin HEAD:master
        echo "$pkg: pushed {{ver}}-$rel"
      fi
      rm -rf "$dir"
    done

# Regenerate both AUR packages' .SRCINFO
srcinfo:
    @for p in darwan darwan-bin; do (cd packaging/aur/$p && makepkg --printsrcinfo > .SRCINFO); done

# Remove build output
clean:
    cargo clean
    rm -rf dist
