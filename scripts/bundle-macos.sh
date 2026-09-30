#!/bin/sh
# release ビルドから最小限の macOS .app bundle（Intel / Apple Silicon の universal、署名なし）を作る。
# 必要なターゲット: rustup target add aarch64-apple-darwin x86_64-apple-darwin
set -eu

cd "$(dirname "$0")/.."
cargo build --release --target aarch64-apple-darwin
cargo build --release --target x86_64-apple-darwin

APP=target/release/DailyWorkMemo.app
VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)

rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS"
lipo -create \
  target/aarch64-apple-darwin/release/daily-work-memo \
  target/x86_64-apple-darwin/release/daily-work-memo \
  -output "$APP/Contents/MacOS/daily-work-memo"

cat > "$APP/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>Daily Work Memo</string>
  <key>CFBundleDisplayName</key><string>Daily Work Memo</string>
  <key>CFBundleIdentifier</key><string>daily-work-memo</string>
  <key>CFBundleExecutable</key><string>daily-work-memo</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>${VERSION}</string>
  <key>CFBundleVersion</key><string>${VERSION}</string>
  <key>NSHighResolutionCapable</key><true/>
  <key>LSMinimumSystemVersion</key><string>11.0</string>
</dict>
</plist>
EOF

echo "$APP"
