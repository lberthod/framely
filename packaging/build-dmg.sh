#!/usr/bin/env bash
# Construit Framely.app puis un .dmg, à partir d'un build release.
#
# Ce script produit un bundle et un .dmg utilisables localement (signature
# ad-hoc, pas de certificat requis). Pour une distribution publique, il
# faudra en plus :
#   1. Signer avec un certificat "Developer ID Application" (nécessite un
#      compte Apple Developer payant) :
#        codesign --force --deep --options runtime \
#          --sign "Developer ID Application: VOTRE NOM (TEAMID)" \
#          --entitlements packaging/entitlements.plist Framely.app
#   2. Notariser auprès d'Apple :
#        xcrun notarytool submit Framely.dmg --keychain-profile "framely" --wait
#        xcrun stapler staple Framely.dmg
# Ces deux étapes nécessitent les identifiants Apple Developer du compte qui
# publiera l'app — je ne peux pas les fournir ni les exécuter à ta place.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_NAME="Framely"
BUNDLE_ID="com.framely.app"
BUILD_DIR="$ROOT_DIR/target/release"
STAGING_DIR="$ROOT_DIR/target/dmg-staging"
APP_DIR="$STAGING_DIR/$APP_NAME.app"
DMG_PATH="$ROOT_DIR/target/$APP_NAME.dmg"

echo "==> Build release"
cargo build --release --manifest-path "$ROOT_DIR/Cargo.toml" --bin framely

echo "==> Assemblage du bundle .app"
rm -rf "$STAGING_DIR"
mkdir -p "$APP_DIR/Contents/MacOS" "$APP_DIR/Contents/Resources"

cp "$BUILD_DIR/framely" "$APP_DIR/Contents/MacOS/framely"
cp "$ROOT_DIR/packaging/Info.plist" "$APP_DIR/Contents/Info.plist"
if [ -f "$ROOT_DIR/packaging/AppIcon.icns" ]; then
  cp "$ROOT_DIR/packaging/AppIcon.icns" "$APP_DIR/Contents/Resources/AppIcon.icns"
else
  echo "!! packaging/AppIcon.icns absent — génère-le d'abord avec :"
  echo "   cargo run -p framely-render --example generate_icon"
  echo "   iconutil -c icns packaging/AppIcon.iconset -o packaging/AppIcon.icns"
fi

echo "==> Signature ad-hoc (pas de certificat requis, suffisant en local)"
codesign --force --deep --sign - "$APP_DIR"

echo "==> Génération du .dmg"
rm -f "$DMG_PATH"
hdiutil create -volname "$APP_NAME" \
  -srcfolder "$STAGING_DIR" \
  -ov -format UDZO \
  "$DMG_PATH"

echo "==> Terminé : $DMG_PATH"
echo "Bundle id : $BUNDLE_ID (placeholder — à changer une fois le nom/domaine définitifs choisis)"
