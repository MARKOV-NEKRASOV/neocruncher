#!/usr/bin/env bash
set -euo pipefail

# Run inside the pinned Ubuntu 24.04 build container with /project, /release,
# and /out mounted. Only CPU calculation is qualified in this trial image.
if ! command -v cargo >/dev/null; then
  curl --proto '=https' --tlsv1.2 -fsSL https://sh.rustup.rs \
    | sh -s -- -y --profile minimal --default-toolchain 1.99.1
fi
python3 -m venv /tmp/neocruncher-packaging-venv
/tmp/neocruncher-packaging-venv/bin/python -m pip install --upgrade pip
mkdir -p /tmp/neocruncher-sources
cp -a /release/Kolmogorov-Engine-1.0.0a1 /release/NeoCruncher-1.0.0a1 /tmp/neocruncher-sources/
/tmp/neocruncher-packaging-venv/bin/python -m pip install \
  pyinstaller /tmp/neocruncher-sources/Kolmogorov-Engine-1.0.0a1 \
  /tmp/neocruncher-sources/NeoCruncher-1.0.0a1

cd /project/gui
export CARGO_TARGET_DIR=/tmp/neocruncher-gui-target
cargo test --locked
cargo build --locked --release

cd /project
/tmp/neocruncher-packaging-venv/bin/pyinstaller \
  --noconfirm --clean --onedir --name neocruncher-cli \
  --collect-all neocruncher_app --collect-all kolmogorov \
  --collect-all pykolm gui/packaging/cli_entry.py \
  --distpath /tmp/neocruncher-pyinstaller-dist \
  --workpath /tmp/neocruncher-pyinstaller-work \
  --specpath /tmp/neocruncher-pyinstaller-spec

rm -rf /out/AppDir
mkdir -p /out/AppDir/usr/bin /out/AppDir/usr/lib
cp /tmp/neocruncher-gui-target/release/neocruncher-gui /out/AppDir/usr/bin/
cp -a /tmp/neocruncher-pyinstaller-dist/neocruncher-cli /out/AppDir/usr/lib/
mkdir -p /out/AppDir/usr/share/doc/neocruncher
cp LICENSE /out/AppDir/usr/share/doc/neocruncher/GUI-LICENSE
cp /release/COPYING /out/AppDir/usr/share/doc/neocruncher/CLI-COPYING
cp /release/NOTICE /out/AppDir/usr/share/doc/neocruncher/CLI-NOTICE
cp /usr/lib/x86_64-linux-gnu/libxkbcommon-x11.so.0 /out/AppDir/usr/lib/
cp /usr/lib/x86_64-linux-gnu/libxcb-xkb.so.1 /out/AppDir/usr/lib/
cp assets/neocruncher-mark-transparent-candidate.png /out/AppDir/neocruncher.png
ln -s neocruncher.png /out/AppDir/.DirIcon
cat > /out/AppDir/neocruncher.desktop <<'EOF'
[Desktop Entry]
Type=Application
Name=NeoCruncher GUI (trial 0.1.3)
Comment=Research frontend for high-precision mathematical constants
Exec=neocruncher-gui
Icon=neocruncher
Categories=Science;Math;
Terminal=false
EOF
cp gui/packaging/AppRun /out/AppDir/AppRun
cp gui/packaging/install-desktop.sh /out/AppDir/install-desktop.sh
chmod +x /out/AppDir/AppRun /out/AppDir/install-desktop.sh
desktop-file-validate /out/AppDir/neocruncher.desktop

# Exercise the frozen computation program before making an AppImage.
/out/AppDir/usr/lib/neocruncher-cli/neocruncher-cli \
  --constant pi --digits 1000 --verify --output /tmp/neocruncher-frozen-pi.txt
test -s /tmp/neocruncher-frozen-pi.txt
test -s /tmp/neocruncher-frozen-pi.txt.json

ldd /out/AppDir/usr/bin/neocruncher-gui > /out/gui-ldd.txt
readelf -V /out/AppDir/usr/bin/neocruncher-gui > /out/gui-versions.txt
chmod -R a+rX /out/AppDir
