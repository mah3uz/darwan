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

# Regenerate both AUR packages' .SRCINFO
srcinfo:
    @for p in darwan darwan-bin; do (cd packaging/aur/$p && makepkg --printsrcinfo > .SRCINFO); done

# Remove build output
clean:
    cargo clean
    rm -rf dist
