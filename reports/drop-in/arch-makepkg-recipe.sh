#!/bin/bash
# Tier-3 ecosystem source-build: Arch's OWN packaging machinery (makepkg + a PKGBUILD) builds zic-rs
# FROM the crates.io source crate, producing a real .pkg.tar.zst; the packaged binary then runs the
# drop-in matrix. Runs INSIDE archlinux:latest WITH network (makepkg fetches the crate + cargo deps).
set -e
echo "## arch: $(grep PRETTY_NAME /etc/os-release | cut -d= -f2)"
echo "## installing base-devel + rust (Arch's own toolchain) ..."
pacman -Sy --noconfirm --quiet base-devel rust sudo >/dev/null 2>&1
echo "## toolchain identity:"
echo "   rustc=$(rustc --version)"
echo "   cargo=$(cargo --version)"
echo "   pacman rust=$(pacman -Q rust 2>/dev/null)"
echo "   makepkg=$(makepkg --version | head -1)"
echo "   glibc=$(ldd --version | head -1)"
# makepkg refuses to run as root -> create an unprivileged build user
useradd -m builder 2>/dev/null || true
install -d -o builder /home/builder/build
cat > /home/builder/build/PKGBUILD <<'PKGB'
pkgname=zic-rs
pkgver=0.1.0
pkgrel=1
pkgdesc="Memory-safe Rust IANA tzdata -> TZif (RFC 9636) compiler"
arch=('x86_64')
url="https://github.com/infinityabundance/zic-rs"
license=('MIT OR Apache-2.0')
makedepends=('cargo')
# fetch the published crate SOURCE from crates.io (renamed so makepkg extracts the .tar.gz)
source=("$pkgname-$pkgver.tar.gz::https://static.crates.io/crates/$pkgname/$pkgname-$pkgver.crate")
sha256sums=('2859db2c57a72086dfe919974c8cab3de7e70160e87a2b8ac61ab25f6bc5e556')
build() {
  cd "$srcdir/$pkgname-$pkgver"
  cargo build --release --locked
}
package() {
  cd "$srcdir/$pkgname-$pkgver"
  install -Dm755 target/release/zic-rs "$pkgdir/usr/bin/zic-rs"
}
PKGB
chown builder:builder /home/builder/build/PKGBUILD
echo "## running makepkg as unprivileged 'builder' (fetch crate -> cargo build --release --locked) ..."
sudo -u builder bash -c 'cd ~/build && makepkg -f --noconfirm' >/tmp/makepkg.log 2>&1 || { echo "MAKEPKG FAILED"; tail -25 /tmp/makepkg.log; exit 1; }
PKG=$(ls /home/builder/build/zic-rs-0.1.0-1-x86_64.pkg.tar.zst 2>/dev/null)
echo "## built Arch package: $(basename "$PKG")  ($(stat -c%s "$PKG") bytes)"
echo "   pkg_sha256=$(sha256sum "$PKG" | cut -d' ' -f1)"
echo "## installing the package (pacman -U) ..."
pacman -U --noconfirm "$PKG" >/dev/null 2>&1
BIN=/usr/bin/zic-rs
echo "## packaged binary:"
echo "   $($BIN --version 2>&1 | head -1)"
echo "   binary_sha256=$(sha256sum $BIN | cut -d' ' -f1)"
echo "   runtime_abi: $(ldd $BIN 2>&1 | head -1)"
echo "## drop-in matrix with the makepkg-built, pacman-installed binary:"
$BIN compile --all-supported --input /src/tzdata.zi --out /out >/dev/null 2>&1; echo "   compile_exit=$?"
echo "   files=$(find /out -type f | wc -l)"
echo "   bundle_hash: $($BIN size-report --out /out --format json 2>/dev/null | grep -oE '\"bundle_hash\": \"[0-9a-f]{16}')"
echo "## tzdata source sha: $(sha256sum /src/tzdata.zi | cut -c1-16)"
echo "## DONE"
