#!/usr/bin/env bash
# Package an already signed KhianKrub.app as a drag-to-Applications disk image.
# Usage: bash packaging/macos/create-dmg.sh /path/to/KhianKrub.app [output.dmg]
set -euo pipefail
. "$(dirname "${BASH_SOURCE[0]}")/../env.sh"

APP="${1:?usage: create-dmg.sh /path/to/KhianKrub.app [output.dmg]}"
OUTPUT="${2:-$DIST/KhianKrub-$VERSION-macos-universal.dmg}"
[ "$(basename "$APP")" = "KhianKrub.app" ] || { echo "error: expected KhianKrub.app" >&2; exit 2; }
codesign --verify --strict --deep "$APP"
test "$(/usr/libexec/PlistBuddy -c 'Print CFBundleShortVersionString' "$APP/Contents/Info.plist")" = "${VERSION%%-*}"

mkdir -p "$CARGO_TARGET_DIR"
WORK="$(mktemp -d "$CARGO_TARGET_DIR/khiankrub-dmg.XXXXXX")"
MOUNT="$WORK/mount"
ATTACHED=0
cleanup() {
  if [ "$ATTACHED" = 1 ]; then hdiutil detach "$MOUNT" -quiet || true; fi
  rm -rf "$WORK"
}
trap cleanup EXIT
mkdir -p "$WORK/stage" "$MOUNT" "$(dirname "$OUTPUT")"
ditto --norsrc --noextattr "$APP" "$WORK/stage/KhianKrub.app"
ln -s /Applications "$WORK/stage/Applications"
hdiutil create -volname KhianKrub -srcfolder "$WORK/stage" -format UDZO -ov "$OUTPUT"
hdiutil verify "$OUTPUT"
hdiutil attach "$OUTPUT" -readonly -nobrowse -mountpoint "$MOUNT" -quiet
ATTACHED=1
codesign --verify --strict --deep "$MOUNT/KhianKrub.app"
test "$(readlink "$MOUNT/Applications")" = /Applications
test "$(/usr/libexec/PlistBuddy -c 'Print CFBundleShortVersionString' "$MOUNT/KhianKrub.app/Contents/Info.plist")" = "${VERSION%%-*}"
hdiutil detach "$MOUNT" -quiet
ATTACHED=0
echo "==> verified: $OUTPUT"
