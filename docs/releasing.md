# Releasing

For maintainers. A release is a git tag, a GitHub Release with the prebuilt package attached, and the two AUR packages:
[`darwan`](https://aur.archlinux.org/packages/darwan) builds the tag's source, and
[`darwan-bin`](https://aur.archlinux.org/packages/darwan-bin) installs the attached package. The AUR files live in
[`packaging/aur/`](../packaging/aur); `just aur` publishes them, so no AUR clone is kept anywhere.

## Once: AUR access

Create an account on [aur.archlinux.org](https://aur.archlinux.org), then a key only for the AUR:

```sh
ssh-keygen -f ~/.ssh/aur
```

Paste `~/.ssh/aur.pub` into *My Account* on the AUR, and add this to `~/.ssh/config`:

```
Host aur.archlinux.org
  IdentityFile ~/.ssh/aur
  User aur
```

AUR commits carry your global git name and email and can't be changed after pushing, so check `git config user.email`.

## A release

The release notes are the `## Unreleased` section of [`CHANGELOG.md`](../CHANGELOG.md), filled in as changes are
committed: what changed for the people using Darwan, and an "Upgrading from" section for anything they have to do.
Read it through before releasing; `just release-notes Unreleased` prints it.

Then one command does all of the steps below:

```sh
just ship 0.1.1
```

It first checks that you're on an up-to-date, clean `main`, that the version is new, that `## Unreleased` isn't empty, that `gh` is logged in, and that
`just lint` and `just test` pass on a fresh build of Darwan's own crates (dependencies stay cached). Then it commits the version bump and asks once before pushing anything, since from
there on the release is public. If a later step fails, it stops and you finish the rest by hand, from the step below
where it stopped.

The steps it runs:

**1. Set the version** in three places: `version` in [`Cargo.toml`](../Cargo.toml), and `pkgver` in
[`packaging/aur/darwan/PKGBUILD`](../packaging/aur/darwan/PKGBUILD) and
[`packaging/aur/darwan-bin/PKGBUILD`](../packaging/aur/darwan-bin/PKGBUILD). Reset both `pkgrel` to `1`. In
`CHANGELOG.md`, add `## 0.1.0 - <date and time>` (like `2026-09-29 18:38 +06:00`) under `## Unreleased`, so what was unreleased becomes this release's
section. Commit it.

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

**4. Publish the GitHub Release** with the package attached and the release's changelog section as its notes:

```sh
just release-notes 0.1.0 > dist/notes-0.1.0.md
gh release create v0.1.0 dist/darwan-0.1.0-x86_64.pkg.tar.zst --title v0.1.0 --notes-file dist/notes-0.1.0.md
```

To correct a published release's notes, fix its section in `CHANGELOG.md`, print it again with `just release-notes`,
and publish it with `gh release edit v0.1.0 --notes-file dist/notes-0.1.0.md`; only the text changes.

**5. Commit the checksums** that step 3 filled in:

```sh
git commit -am "Release 0.1.0"
git push
```

**6. Publish to the AUR:**

```sh
just aur
```

For each package it clones the AUR repository into `dist/aur/`, copies in `PKGBUILD`, `.SRCINFO` and `LICENSE`,
commits "Update to 0.1.0-1", pushes to `master` and deletes the clone. It stops before pushing anything if
`packaging/aur` has uncommitted changes, a checksum is still `SKIP`, a `.SRCINFO` doesn't match its PKGBUILD, or the
GitHub Release has no package for `darwan-bin` to download. A package whose files haven't changed is skipped.

## A packaging fix without a release

Bump `pkgrel` in the PKGBUILD you changed, then:

```sh
just srcinfo
git commit -am "darwan: <what changed>"
git push
just aur
```

The AUR shows what `.SRCINFO` says, not the PKGBUILD, so never skip `just srcinfo`.
