#!/usr/bin/env bash
#
# Keep crates/cargo-gears/lints.tree in sync with crates/cargo-gears-lints.
#
# cargo-gears-lints isn't published - the CLI fetches it from this repository at its
# own release tag. release-plz only sees commits that touch files packaged with
# cargo-gears, so the marker records the git tree id of the lints sources. Every
# lints change then also changes a cargo-gears file, which bumps its version and
# lands the commit in CHANGELOG.md.
#
# The tree id is taken from the index, so stage the lints changes first.
#
# Usage: make lints-tree | make lints-tree-check
#        (or: bash tools/scripts/lints-tree.sh update|check)

set -euo pipefail

root="$(git rev-parse --show-toplevel)"
marker="crates/cargo-gears/lints.tree"
lints="crates/cargo-gears-lints"

expected="$(git -C "$root" write-tree --prefix="$lints/")"

case "${1:-}" in
  update)
    printf '%s\n' "$expected" > "$root/$marker"
    echo "$marker: $expected"
    ;;
  check)
    actual="$(cat "$root/$marker" 2>/dev/null || true)"
    if [[ "$actual" != "$expected" ]]; then
      echo "error: $marker is out of date (expected $expected, found ${actual:-nothing})." >&2
      echo "Stage the $lints changes and run \`make lints-tree\`." >&2
      exit 1
    fi
    echo "$marker is up to date."
    ;;
  *)
    echo "Usage: $0 update|check" >&2
    exit 2
    ;;
esac
