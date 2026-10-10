#!/usr/bin/env bash
# Build a Lootsmith AppImage (run from the repo root, Linux, with cargo).
#
#   packaging/appimage/build.sh [out-dir]
#
# Uses linuxdeploy (downloaded on first run) to assemble the AppImage from the
# cargo release build plus the in-repo .desktop and icon.
set -euo pipefail

out="${1:-dist}"
version=$(awk -F'"' '/^version = /{print $2; exit}' Cargo.toml)
mkdir -p "$out/AppDir/usr/bin" "$out/AppDir/usr/share/applications" "$out/AppDir/usr/share/icons/hicolor/256x256/apps"

echo "building (release)…"
cargo build --release -p lootsmith-gui -p lootsmith-cli
cp target/release/lootsmith-gui target/release/lootsmith "$out/AppDir/usr/bin/"
cp packaging/appimage/lootsmith.desktop "$out/AppDir/usr/share/applications/"
cp packaging/appicon/lootsmith.png \
   "$out/AppDir/usr/share/icons/hicolor/256x256/apps/" 2>/dev/null || true
cp packaging/appicon/lootsmith.png "$out/AppDir/lootsmith.png" 2>/dev/null || true
cat > "$out/AppDir/AppRun" << 'RUNEOF'
#!/bin/sh
exec "$(dirname "$0")/usr/bin/lootsmith-gui" "$@"
RUNEOF
chmod +x "$out/AppDir/AppRun"

cd "$out"
if [ ! -x linuxdeploy ]; then
  curl -sL https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage -o linuxdeploy
  chmod +x linuxdeploy
fi
./linuxdeploy --appdir AppDir --output appimage
cd -

echo "wrote $out/Lootsmith-${version}-x86_64.AppImage"
