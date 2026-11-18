#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 2 ]]; then
  echo "Usage: $0 BINARY GLIBC_FLOOR" >&2
  exit 2
fi

binary=$1
floor=$2
if [[ ! -f "$binary" ]]; then
  echo "ADR 0103 Decision 4: binary does not exist: $binary" >&2
  exit 1
fi
if [[ ! "$floor" =~ ^[0-9]+\.[0-9]+$ ]]; then
  echo "ADR 0103 Decision 4: invalid glibc floor: $floor" >&2
  exit 2
fi

latest=$(objdump -T "$binary" | sed -nE 's/.*GLIBC_([0-9]+\.[0-9]+).*/\1/p' | sort -Vu | tail -n 1)
if [[ -z "$latest" ]]; then
  echo "ADR 0103 Decision 4: binary has no GLIBC symbol versions" >&2
  exit 1
fi

if [[ "$(printf '%s\n' "$floor" "$latest" | sort -V | tail -n 1)" != "$floor" ]]; then
  echo "ADR 0103 Decision 4: binary needs GLIBC_$latest, above floor $floor" >&2
  exit 1
fi
