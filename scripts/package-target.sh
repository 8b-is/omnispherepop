#!/usr/bin/env bash
# Package ospop binaries for one target triple. Cross-compiled binaries are
# packaged but not executed here; only host smoke tests run.
set -euo pipefail
target="$1"
root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

case "$target" in
  *windows*) exe=.exe; ext=zip ;;
  *) exe=; ext=tar.gz ;;
esac

mkdir -p "dist/$target"
stage="dist/$target/omnispherepop"
rm -rf "$stage"
mkdir -p "$stage"
cp "target/$target/release/ospop$exe" "$stage/" 2>/dev/null || cp "target/release/ospop$exe" "$stage/"
cp "target/$target/release/ospop-mcp$exe" "$stage/" 2>/dev/null || cp "target/release/ospop-mcp$exe" "$stage/"
cp README.md LICENSE-MIT LICENSE-APACHE "$stage/"

if [[ "$ext" == "zip" ]]; then
  if command -v zip >/dev/null 2>&1; then
    (cd "dist/$target" && zip -qr "omnispherepop-$target.zip" omnispherepop)
  elif command -v powershell.exe >/dev/null 2>&1; then
    # Windows runners have no `zip`; Compress-Archive is the equivalent.
    powershell.exe -NoProfile -Command \
      "Compress-Archive -Path 'dist/$target/omnispherepop' -DestinationPath 'dist/$target/omnispherepop-$target.zip' -Force"
  else
    printf 'no zip tool available for %s\n' "$target" >&2
    exit 1
  fi
else
  tar -czf "dist/$target/omnispherepop-$target.tar.gz" -C "dist/$target" omnispherepop
fi
rm -rf "$stage"
printf 'packaged dist/%s\n' "$target"