# Development

Building Darwan from source, running it without installing, testing it, and publishing a release.

## What you need

Everything the package needs at runtime (see the [README](../README.md#installation)), plus:

```sh
sudo pacman -S --needed rust lld librsvg just
```

For the offscreen theme checks and `preview --at`, also:

```sh
sudo pacman -S --needed libfaketime
```

Then get the code:

```sh
git clone https://github.com/mah3uz/darwan.git
cd darwan
```

## Run it without installing

The [`justfile`](../justfile) points every binary at this checkout's `runtime/` and `themes/` (through
`DARWAN_DATA_DIR`), so nothing needs installing. Run `just` on its own to list every recipe.

| Recipe                        | Does                                                                 |
|:------------------------------|:---------------------------------------------------------------------|
| `just darwan`                 | opens the TUI                                                        |
| `just darwan list`            | runs any CLI command; everything after `darwan` is passed through    |
| `just gui`                    | opens the GUI                                                        |
| `just preview pixel-coffee`   | full-screen preview; add flags after the theme id                    |

Applying a theme to SDDM and importing fonts go through the privileged helper, which only works once installed.

## Test

| Recipe                  | Does                                                                            |
|:------------------------|:--------------------------------------------------------------------------------|
| `just test`             | Rust, QML and theme-lint tests                                                  |
| `just lint`             | `cargo fmt --check` and clippy with warnings as errors                          |
| `just check`            | loads every theme offscreen, fails on any QML warning, and types the password   |
| `just check --no-fonts` | the same, with each theme's `font/` hidden, as on a fresh clone                 |

Run all three before sending a change:

```sh
just test lint check
```

Writing or changing a theme? [The theme contract](./theme-contract.md) lists what a theme can rely on and the rules
`just check` enforces.

## Build and install a package

This packs the committed `HEAD` the way GitHub packs a release tag, builds it with the AUR `darwan` PKGBUILD (tests
included) and installs it. Uncommitted changes are left out.

```sh
just install
```

It asks for your password twice: once if `makepkg` needs to install a build dependency, and once for `pacman`. To
remove it again:

```sh
just uninstall
```

## Release

A release is a git tag, a GitHub Release with the prebuilt package attached, and the two AUR packages:
[`darwan`](https://aur.archlinux.org/packages/darwan) builds the tag's source, and
[`darwan-bin`](https://aur.archlinux.org/packages/darwan-bin) installs the attached package.

**1. Set the version** in three places: `version` in [`Cargo.toml`](../Cargo.toml), and `pkgver` in
[`packaging/aur/darwan/PKGBUILD`](../packaging/aur/darwan/PKGBUILD) and
[`packaging/aur/darwan-bin/PKGBUILD`](../packaging/aur/darwan-bin/PKGBUILD). Reset both `pkgrel` to `1`. Commit it.

**2. Tag and push:**

```sh
git tag v0.1.0
git push origin main v0.1.0
```

**3. Build the release.** This downloads the tag's tarball, fills in both PKGBUILDs' checksums, builds the package into
`dist/` and regenerates both `.SRCINFO` files:

```sh
just release
```

**4. Publish the GitHub Release** with the package attached:

```sh
gh release create v0.1.0 dist/darwan-0.1.0-x86_64.pkg.tar.zst --title v0.1.0 --generate-notes
```

**5. Commit the checksums** that step 3 filled in:

```sh
git commit -am "Release 0.1.0"
git push
```

**6. Update the AUR packages.** Each AUR package is its own git repository. The first time, clone both next to this
checkout (this needs an AUR account with your SSH key). The AUR only accepts pushes to `master`, so the clone must not
take your `init.defaultBranch`:

```sh
git -c init.defaultBranch=master clone ssh://aur@aur.archlinux.org/darwan.git ../aur-darwan
git -c init.defaultBranch=master clone ssh://aur@aur.archlinux.org/darwan-bin.git ../aur-darwan-bin
```

Then, for every release, copy the files across and push. For `darwan`:

```sh
cp packaging/aur/darwan/{PKGBUILD,.SRCINFO,LICENSE} ../aur-darwan/
git -C ../aur-darwan commit -am "Update to 0.1.0"
git -C ../aur-darwan push
```

And for `darwan-bin`:

```sh
cp packaging/aur/darwan-bin/{PKGBUILD,.SRCINFO,LICENSE} ../aur-darwan-bin/
git -C ../aur-darwan-bin commit -am "Update to 0.1.0"
git -C ../aur-darwan-bin push
```

The first push to a new AUR repository needs `git add PKGBUILD .SRCINFO LICENSE` before the commit, since `-a` only
picks up files git already tracks. `LICENSE` covers the packaging files themselves; the AUR asks every package
repository for one.

A fix to packaging alone, with no new release, bumps `pkgrel` instead of `pkgver` and skips steps 2 to 4. Run
`just srcinfo` after editing a PKGBUILD, since the AUR shows what `.SRCINFO` says, not the PKGBUILD.

## Demo animations

The animations in the README's gallery live in [darwan-assets](https://github.com/mah3uz/darwan-assets), which also
holds the tools that record them and how to run those tools.
