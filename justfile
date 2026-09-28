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

# The whole release: version bump, tag, GitHub Release and AUR, e.g. `just ship 0.2.1`
ship version:
    #!/usr/bin/env bash
    set -euo pipefail
    v={{version}}
    old=$(sed -n 's/^pkgver=//p' packaging/aur/darwan/PKGBUILD)
    fail() { echo "ship: $*" >&2; exit 1; }

    [[ $v =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || fail "version must look like 0.2.1, not $v"
    [[ $(printf '%s\n%s\n' "$old" "$v" | sort -V | tail -1) == "$v" && $v != "$old" ]] || fail "$v is not newer than $old"
    [[ $(git branch --show-current) == main ]] || fail "switch to main first"
    [[ -z $(git status --porcelain) ]] || fail "commit or stash your changes first"
    git fetch -q origin
    [[ $(git rev-list --count HEAD..origin/main) == 0 ]] || fail "main is behind origin/main; pull first"
    ! git rev-parse -q --verify "refs/tags/v$v" >/dev/null || fail "tag v$v already exists"
    ! git ls-remote --exit-code --tags origin "v$v" >/dev/null || fail "tag v$v already exists on origin"
    gh auth status >/dev/null 2>&1 || fail "gh is not logged in; run gh auth login"

    echo "==> lint and tests"
    just lint
    just test

    echo "==> version $old -> $v"
    sed -i "0,/^version = \"$old\"/s//version = \"$v\"/" Cargo.toml
    for p in packaging/aur/darwan/PKGBUILD packaging/aur/darwan-bin/PKGBUILD; do
      sed -i "s/^pkgver=.*/pkgver=$v/; s/^pkgrel=.*/pkgrel=1/" "$p"
    done
    cargo update --workspace -q
    git commit -q -am "Version $v"

    # Everything after this is public and can't be taken back.
    read -rp "Push v$v to origin, GitHub and the AUR? [y/N] " answer
    if [[ $answer != [yY] ]]; then
      echo "Stopped before pushing. To undo the version commit: git reset --hard HEAD~1"
      exit 1
    fi
    trap 'echo "ship: stopped; finish the remaining steps in docs/releasing.md by hand" >&2' ERR

    echo "==> tag and push"
    git tag "v$v"
    git push origin main "v$v"

    echo "==> checksums and package"
    DARWAN_SHIP=1 packaging/release.sh

    echo "==> GitHub Release"
    gh release create "v$v" "dist/darwan-$v-x86_64.pkg.tar.zst" --title "v$v" --generate-notes

    echo "==> commit the checksums"
    git commit -q -am "Release $v"
    git push origin main

    echo "==> AUR"
    just aur
    echo "Released $v."

# Regenerate both AUR packages' .SRCINFO
srcinfo:
    @for p in darwan darwan-bin; do (cd packaging/aur/$p && makepkg --printsrcinfo > .SRCINFO); done

# Remove build output
clean:
    cargo clean
    rm -rf dist
