#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

target_dir=$(cargo metadata --format-version 1 --no-deps | python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])')
version=$(cargo metadata --format-version 1 --no-deps | python3 -c 'import json,sys; print(json.load(sys.stdin)["packages"][0]["version"])')
cargo build --release
app="dist/mdview.app"
rm -rf "$app"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
cp "$target_dir/release/mdview" "$app/Contents/MacOS/mdview"
sed "s/__VERSION__/$version/g" bundle/Info.plist > "$app/Contents/Info.plist"
iconset=$(mktemp -d)/mdview.iconset
swift scripts/make-icon.swift "$iconset"
iconutil -c icns "$iconset" -o "$app/Contents/Resources/mdview.icns"
plutil -lint "$app/Contents/Info.plist" >/dev/null
codesign --force --deep --sign - "$app"
echo "$app"
