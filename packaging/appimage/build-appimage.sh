#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 4 ]]; then
  echo "Usage: $0 BINARY VERSION APPIMAGETOOL RUNTIME" >&2
  exit 2
fi

binary=$1
version=$2
appimagetool=$3
runtime=$4
root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
appdir="$root/target/appimage/HitIt.AppDir"
output="$root/target/HitIt-${version}-x86_64.AppImage"

for path in "$binary" "$appimagetool" "$runtime"; do
  if [[ ! -f "$path" ]]; then
    echo "Required file does not exist: $path" >&2
    exit 1
  fi
done

rm -rf "$appdir"
install -D "$binary" "$appdir/usr/bin/hitit"
install -D "$root/packaging/appimage/AppRun" "$appdir/AppRun"
install -D "$root/packaging/arch/hitit.desktop" "$appdir/hitit.desktop"
install -D "$root/src/assets/music_network_logo.png" \
  "$appdir/hitit.png"
chmod +x "$appdir/AppRun" "$appimagetool"
mkdir -p "$root/target"
"$appimagetool" --runtime-file "$runtime" "$appdir" "$output"
printf '%s\n' "$output"
