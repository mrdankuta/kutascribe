#!/bin/bash
# Build Kutascribe AppImage (Linux): bundles kutascribe-gui + kutascribe-cli + ffmpeg wrapper
# Usage: ./packaging/appimage.sh  (run after cargo build --release)
set -e
APP=Kutascribe
APPDIR=$APP.AppDir
export ARCH=${ARCH:-x86_64}
BIN=${1:-target/release/kutascribe-gui}
CLI=${2:-target/release/kutascribe-cli}

# CI downloads artifacts to dist-bin/<target>-*/target/<target>/release/
if [ ! -f "$BIN" ]; then
  FOUND_APP=$(find dist-bin target -name kutascribe-gui -type f 2>/dev/null | head -1 || true)
  [ -n "$FOUND_APP" ] && BIN="$FOUND_APP"
fi
if [ ! -f "$CLI" ]; then
  FOUND_CLI=$(find dist-bin target -name kutascribe-cli -type f 2>/dev/null | head -1 || true)
  [ -n "$FOUND_CLI" ] && CLI="$FOUND_CLI"
fi
echo "Using BIN=$BIN CLI=$CLI"
mkdir -p $APPDIR/usr/bin
cp $BIN $APPDIR/usr/bin/kutascribe-gui
chmod +x $APPDIR/usr/bin/kutascribe-gui
ln -sf usr/bin/kutascribe-gui $APPDIR/AppRun
cp $CLI $APPDIR/usr/bin/kutascribe-cli 2>/dev/null || echo "cli not found, skipping"

cat > $APPDIR/kutascribe.desktop <<'DESK'
[Desktop Entry]
Name=Kutascribe
Exec=kutascribe-gui
Icon=kutascribe
Type=Application
Categories=AudioVideo;Utility;
Comment=Local speech-to-text (audio+video to .txt)
DESK

# minimal 1x1 png icon (replace with real logo later)
mkdir -p $APPDIR
echo 'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==' | base64 -d > $APPDIR/kutascribe.png
cp $APPDIR/kutascribe.png $APPDIR/.DirIcon
mkdir -p $APPDIR/usr/share/icons/hicolor/256x256/apps
cp $APPDIR/kutascribe.png $APPDIR/usr/share/icons/hicolor/256x256/apps/kutascribe.png

# appimagetool
if ! command -v appimagetool >/dev/null; then
  wget -q https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage -O /tmp/appimagetool
  chmod +x /tmp/appimagetool
  APPIMAGETOOL=/tmp/appimagetool
else
  APPIMAGETOOL=appimagetool
fi

$APPIMAGETOOL $APPDIR ${APP}-x86_64.AppImage
echo "Built ${APP}-x86_64.AppImage"
echo "Requires on host: ffmpeg, Vulkan loader (usually via GPU drivers). Models download to ~/.cache/kutascribe on first run."
