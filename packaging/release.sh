#!/usr/bin/env bash
# Run after pushing the tag v<pkgver>: fills both AUR PKGBUILDs' checksums and .SRCINFO files and
# builds the release asset for darwan-bin. It uploads and pushes nothing.
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
aur=$root/packaging/aur
ver=$(sed -n 's/^pkgver=//p' "$aur/darwan/PKGBUILD")
url=$(sed -n "s/^url='\(.*\)'/\1/p" "$aur/darwan/PKGBUILD")
dist=$root/dist
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

set_sum() {
  sed -i "s/^sha256sums=('[^']*')/sha256sums=('$2')/" "$1"
}

echo "==> darwan $ver: checksum of the v$ver tarball"
curl -fsSL "$url/archive/refs/tags/v$ver.tar.gz" -o "$work/darwan-$ver.tar.gz"
set_sum "$aur/darwan/PKGBUILD" "$(sha256sum "$work/darwan-$ver.tar.gz" | cut -d' ' -f1)"

echo "==> building the package from the tag"
cp "$aur/darwan/PKGBUILD" "$work/"
(cd "$work" && makepkg -f --noconfirm)
mkdir -p "$dist"
asset=$dist/darwan-$ver-x86_64.pkg.tar.zst
cp "$work"/darwan-"$ver"-*-x86_64.pkg.tar.zst "$asset"

echo "==> darwan-bin: checksum of the release asset"
set_sum "$aur/darwan-bin/PKGBUILD" "$(sha256sum "$asset" | cut -d' ' -f1)"

for pkg in darwan darwan-bin; do
  (cd "$aur/$pkg" && makepkg --printsrcinfo > .SRCINFO)
done

cat <<EOF

Done. Next, by hand:
  gh release create v$ver "$asset" --title "v$ver"     # or: gh release upload v$ver "$asset"
  git commit -am "Release $ver" && git push
  just aur
EOF
