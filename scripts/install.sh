#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

destination="${MDVIEW_INSTALL_DIR:-$HOME/Applications}"
lsregister=/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister
scripts/bundle.sh
mkdir -p "$destination"
pkill -x mdview 2>/dev/null || true
rm -rf "$destination/mdview.app"
cp -R dist/mdview.app "$destination/mdview.app"
"$lsregister" -u "$PWD/dist/mdview.app" 2>/dev/null || true
"$lsregister" -f -R -trusted "$destination/mdview.app"
swift scripts/set-default.swift "$destination/mdview.app"
