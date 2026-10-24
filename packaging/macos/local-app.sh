#!/usr/bin/env bash
# Assemble a double-clickable KhianKrub.app from the release binary you already built (ad-hoc
# signed; no DMG, no notarization). Release zips are made from it too.
#
#   cargo build --release -p wordcraft                    # build (or rebuild) the binary first
#   packaging/macos/local-app.sh [DEST]                   # DEST defaults to ~/Applications/KhianKrub.app
#
# Universal (Apple silicon and Intel in one app), as the release zips are:
#
#   MACOSX_DEPLOYMENT_TARGET=11.0 cargo build --release -p wordcraft \
#     --target aarch64-apple-darwin --target x86_64-apple-darwin
#   packaging/macos/local-app.sh --universal [DEST]
#
# Run it again after each rebuild to refresh the app. The steps mirror package.sh.
set -euo pipefail
# shellcheck source=../env.sh
. "$(dirname "${BASH_SOURCE[0]}")/../env.sh"
HERE="$ROOT/packaging/macos"
UNIVERSAL=0
if [ "${1:-}" = "--universal" ]; then
  UNIVERSAL=1
  shift
fi
APP="${1:-$HOME/Applications/KhianKrub.app}"
SHORT_VERSION="${VERSION%%-*}"
SHA="$(git -C "$ROOT" rev-parse --short HEAD 2>/dev/null || echo unknown)"
[ "$(basename "$APP")" = "KhianKrub.app" ] || { echo "error: DEST must end in KhianKrub.app" >&2; exit 2; }

if [ "$UNIVERSAL" = 1 ]; then
  # Both slices joined into one binary, as package.sh does.
  BIN="$CARGO_TARGET_DIR/macos-universal/wordcraft"
  inputs=()
  for t in aarch64-apple-darwin x86_64-apple-darwin; do
    slice="$CARGO_TARGET_DIR/$t/release/wordcraft"
    [ -x "$slice" ] || { echo "error: $slice not found; build both targets first (see the header)" >&2; exit 1; }
    inputs+=("$slice")
  done
  mkdir -p "$(dirname "$BIN")"
  lipo -create -output "$BIN" "${inputs[@]}"
  lipo -info "$BIN"
else
  BIN="$CARGO_TARGET_DIR/release/wordcraft"
  [ -x "$BIN" ] || { echo "error: $BIN not found; run: cargo build --release -p wordcraft" >&2; exit 1; }
fi

echo "==> assembling $APP from $BIN (commit $SHA)"
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp "$BIN" "$APP/Contents/MacOS/KhianKrub"
cp "$ROOT/assets/app-icon/wordcraft.icns" "$APP/Contents/Resources/KhianKrub.icns"
# The licences that must travel with the app: the project's (MIT and Apache-2.0), the NOTICE with its
# third-party credits and the ATTRIBUTION list it points to, and the licences of the embedded fonts (OFL).
cp "$ROOT/LICENSE-MIT" "$ROOT/LICENSE-APACHE" "$ROOT/NOTICE" "$ROOT/ATTRIBUTION.md" "$APP/Contents/Resources/"
cp "$ROOT"/assets/fonts/OFL-*.txt "$ROOT/assets/app-icon/OFL-Sarabun.txt" "$APP/Contents/Resources/"
sed -e "s/@VERSION@/$VERSION/g" -e "s/@SHORT_VERSION@/$SHORT_VERSION/g" -e "s/@BUILD_SHA@/$SHA/g" \
  "$HERE/Info.plist.in" >"$APP/Contents/Info.plist"
plutil -lint "$APP/Contents/Info.plist"
# The name Finder shows: KhianKrub, and เขียนครับ on a Mac set to Thai.
mkdir -p "$APP/Contents/Resources/en.lproj" "$APP/Contents/Resources/th.lproj"
printf '"CFBundleDisplayName" = "KhianKrub";\n"CFBundleName" = "KhianKrub";\n' >"$APP/Contents/Resources/en.lproj/InfoPlist.strings"
printf '"CFBundleDisplayName" = "เขียนครับ";\n"CFBundleName" = "เขียนครับ";\n' >"$APP/Contents/Resources/th.lproj/InfoPlist.strings"
plutil -lint "$APP/Contents/Resources/en.lproj/InfoPlist.strings" "$APP/Contents/Resources/th.lproj/InfoPlist.strings"
printf 'APPL????' >"$APP/Contents/PkgInfo"

# codesign --strict rejects extended attributes; nothing after this may write them onto the bundle.
xattr -cr "$APP"
codesign --force --sign - --timestamp=none --options runtime --entitlements "$HERE/entitlements.plist" "$APP/Contents/MacOS/KhianKrub"
codesign --force --sign - --timestamp=none --options runtime --entitlements "$HERE/entitlements.plist" "$APP"
codesign --verify --strict --deep --verbose=2 "$APP"
# Let Launch Services notice the new icon and document types.
/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister -f "$APP" >/dev/null 2>&1 || true
echo "==> done: $APP"
