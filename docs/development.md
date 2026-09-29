# Development

Building Darwan from source, running it without installing and testing it. Publishing a release is in
[Releasing](./releasing.md).

## What you need

Everything the package needs at runtime (see the [README](../README.md#installation)), plus:

```sh
sudo pacman -S --needed rust lld librsvg cmake just
```

`just build` also builds darwan's QML plugin (`plugin/`, C++ against `mpvqt`) into `target/plugin/`, where a checkout's
binaries find it.

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
| `just tui`                    | opens the TUI                                                        |
| `just darwan list`            | runs any CLI command; everything after `darwan` is passed through    |
| `just gui`                    | opens the GUI                                                        |
| `just preview pixel-coffee`   | full-screen preview; add flags after the theme id                    |
| `just preview osu --saver`    | the screensaver: ambient until a key or click                        |

Applying a theme to SDDM and importing fonts go through the privileged helper, which only works once installed.

## Test

| Recipe                  | Does                                                                            |
|:------------------------|:--------------------------------------------------------------------------------|
| `just test`             | Rust, QML and theme-lint tests                                                  |
| `just lint`             | `cargo fmt --check` and clippy with warnings as errors                          |
| `just check`            | loads every theme offscreen, fails on any QML warning, and types the password, also from the screensaver's ambient mode |
| `just check --no-fonts` | the same, with each theme's `font/` hidden, as on a fresh clone                 |

Run all three before sending a change:

```sh
just test lint check
```

A change people using Darwan will notice (a feature, a fix, changed behaviour, a new setting or dependency) also adds a
line under `## Unreleased` in [`CHANGELOG.md`](../CHANGELOG.md), in the same commit. That section becomes the next
release's notes.

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

## Demo animations

The animations in the README's gallery live in [darwan-assets](https://github.com/mah3uz/darwan-assets), which also
holds the tools that record them and how to run those tools.
