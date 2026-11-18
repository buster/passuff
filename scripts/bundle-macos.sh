#!/usr/bin/env bash
# Build a Passuff.app bundle around an existing passuff binary.
#
# Usage: scripts/bundle-macos.sh [BINARY] [OUTPUT_DIR]
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
binary="${1:-$root/target/release/passuff}"
output="${2:-$root/target/Passuff.app}"

if [[ ! -x "$binary" ]]; then
  printf 'bundle-macos: executable not found: %s\n' "$binary" >&2
  exit 1
fi

mkdir -p "$output/Contents/MacOS"
install -m 755 "$binary" "$output/Contents/MacOS/passuff"

cat > "$output/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDevelopmentRegion</key>
  <string>en</string>
  <key>CFBundleExecutable</key>
  <string>passuff</string>
  <key>CFBundleIdentifier</key>
  <string>dev.passuff.app</string>
  <key>CFBundleName</key>
  <string>passuff</string>
  <key>CFBundleDisplayName</key>
  <string>passuff</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleShortVersionString</key>
  <string>0.1.3</string>
  <key>CFBundleVersion</key>
  <string>0.1.3</string>
  <key>LSUIElement</key>
  <true/>
</dict>
</plist>
PLIST

printf '%s\n' "$output"
