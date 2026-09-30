#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
cargo build -p droidless --release --locked
mkdir -p artifacts/DROIDLESS.app/Contents/MacOS
cp target/release/droidless artifacts/DROIDLESS.app/Contents/MacOS/DROIDLESS
cat > artifacts/DROIDLESS.app/Contents/Info.plist <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleName</key><string>DROIDLESS</string>
<key>CFBundleDisplayName</key><string>DROIDLESS</string>
<key>CFBundleIdentifier</key><string>org.droidless.desktop</string>
<key>CFBundleExecutable</key><string>DROIDLESS</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleVersion</key><string>1</string>
<key>NSHighResolutionCapable</key><true/>
</dict></plist>
PLIST
printf '%s\n' "Built artifacts/DROIDLESS.app (unsigned development bundle)"
