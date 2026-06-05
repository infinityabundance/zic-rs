#!/bin/sh
# Tier-3: RPM-family packaging — a .spec + rpmbuild builds zic-rs FROM the crates.io source crate into a
# real .rpm; rpm -i installs it; the packaged binary runs the drop-in matrix. Runs in almalinux:9.
set -e
echo "## rpm distro: $(grep PRETTY_NAME /etc/os-release | cut -d= -f2)"
echo "## dnf install cargo rust rpm-build ..."
dnf install -y -q cargo rust rpm-build rpmdevtools >/dev/null 2>&1
echo "## toolchain identity:"
echo "   rustc=$(rustc --version 2>&1)"
echo "   cargo=$(cargo --version 2>&1)"
echo "   rpm cargo=$(rpm -q cargo 2>/dev/null)"
echo "   rpmbuild=$(rpmbuild --version 2>&1)"
echo "   glibc=$(ldd --version 2>&1 | head -1)"
rpmdev-setuptree
curl -fsSL -o ~/rpmbuild/SOURCES/zic-rs-0.1.0.crate https://static.crates.io/crates/zic-rs/zic-rs-0.1.0.crate
echo "   crate_sha256=$(sha256sum ~/rpmbuild/SOURCES/zic-rs-0.1.0.crate | cut -d' ' -f1)"
cat > ~/rpmbuild/SPECS/zic-rs.spec <<'SPEC'
%global debug_package %{nil}
Name:           zic-rs
Version:        0.1.0
Release:        1%{?dist}
Summary:        Memory-safe Rust IANA tzdata -> TZif (RFC 9636) compiler
License:        MIT OR Apache-2.0
URL:            https://github.com/infinityabundance/zic-rs
Source0:        https://static.crates.io/crates/zic-rs/zic-rs-%{version}.crate
BuildRequires:  cargo
%description
A reference-admitted Rust TZif compiler candidate, built from the crates.io source crate.
%prep
%setup -q -n zic-rs-%{version}
%build
cargo build --release --locked
%install
install -Dm755 target/release/zic-rs %{buildroot}%{_bindir}/zic-rs
%files
%{_bindir}/zic-rs
SPEC
echo "## rpmbuild -bb (fetch deps + cargo build --release --locked from crate source) ..."
rpmbuild -bb ~/rpmbuild/SPECS/zic-rs.spec >/tmp/rpmbuild.log 2>&1 || { echo "RPMBUILD FAILED"; tail -25 /tmp/rpmbuild.log; exit 1; }
RPM=$(ls ~/rpmbuild/RPMS/x86_64/zic-rs-0.1.0-1*.rpm 2>/dev/null | head -1)
echo "## built RPM: $(basename "$RPM")  ($(stat -c%s "$RPM") bytes)"
echo "   rpm_sha256=$(sha256sum "$RPM" | cut -d' ' -f1)"
echo "## rpm -i installing ..."
rpm -i --replacepkgs "$RPM" >/dev/null 2>&1
BIN=/usr/bin/zic-rs
echo "## packaged binary:"
echo "   $($BIN --version 2>&1 | head -1)"
echo "   binary_sha256=$(sha256sum $BIN | cut -d' ' -f1)"
echo "   runtime_abi: $(ldd $BIN 2>&1 | head -1)"
echo "## drop-in matrix with the rpmbuild-built, rpm-installed binary:"
$BIN compile --all-supported --input /src/tzdata.zi --out /out >/dev/null 2>&1; echo "   compile_exit=$?"
echo "   files=$(find /out -type f | wc -l)"
echo "   bundle_hash: $($BIN size-report --out /out --format json 2>/dev/null | grep -oE '\"bundle_hash\": \"[0-9a-f]{16}')"
echo "## tzdata source sha: $(sha256sum /src/tzdata.zi | cut -c1-16)"
echo "## DONE"
