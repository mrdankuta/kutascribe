#!/bin/bash
# Build Kutascribe AppImage (Linux): bundles kutascribe-app + kutascribe + ffmpeg wrapper
# Usage: ./packaging/appimage.sh  (run after cargo build --release)
set -e
APP=Kutascribe
APPDIR=$APP.AppDir
BIN=${1:-target/release/kutascribe-app}
CLI=${2:-target/release/kutascribe}

mkdir -p $APPDIR/usr/bin
cp $BIN $APPDIR/usr/bin/kutascribe-app
cp $CLI $APPDIR/usr/bin/kutascribe 2>/dev/null || echo "cli not found, skipping"

cat > $APPDIR/kutascribe.desktop <<'DESK'
[Desktop Entry]
Name=Kutascribe
Exec=kutascribe-app
Icon=kutascribe
Type=Application
Categories=AudioVideo;Utility;
Comment=Local speech-to-text (audio+video to .txt)
DESK

# icon placeholder (replace with real png)
mkdir -p $APPDIR/usr/share/icons
echo "P1 1 1 1" > $APPDIR/kutascribe.pbm || true

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
