#!/usr/bin/env bash
#
# Preview or lint the documentation website (gears-rust-web-docs) with the LOCAL
# docs/web-docs content from this cargo-gears checkout — including uncommitted edits.
#
# The web docs site lives in a separate repo (TypeScript/Astro) and syncs content
# from two source repos (gears-rust + cargo-gears). This script clones it into a
# gitignored cache dir, then:
#   dev  — runs the dev server with the FULL site (resolves gears-rust too)
#   lint — syncs only this repo's docs and runs markdownlint on them
#
# Usage: make docs-preview | make docs-lint
#        (or: bash tools/scripts/docs-site.sh dev|lint)

set -euo pipefail

CMD="${1:-dev}"

REPO_ROOT="$(git rev-parse --show-toplevel)"
CACHE_DIR="${GEARS_DOCS_CACHE:-$REPO_ROOT/.web-docs-preview}"
DOCS_REPO="${GEARS_DOCS_REPO:-https://github.com/constructorfabric/gears-rust-web-docs.git}"
# Site branch to use. Override to test an unmerged site PR, e.g.
# GEARS_DOCS_REF=fix/asto-config-new-section make docs-preview
DOCS_REF="${GEARS_DOCS_REF:-main}"

# --- Prerequisite: Node >= 22.13 (required by Astro / the docs site) ---
if ! command -v node >/dev/null 2>&1; then
  echo "ERROR: node not found. Install Node.js >= 22.13 to work with the docs site." >&2
  exit 1
fi

# Try to switch to Node 22 via nvm if current version is too old
node_major="$(node -p 'process.versions.node.split(".")[0]')"
node_minor="$(node -p 'process.versions.node.split(".")[1]')"
if [ "$node_major" -lt 22 ] || { [ "$node_major" -eq 22 ] && [ "$node_minor" -lt 13 ]; }; then
  if command -v nvm >/dev/null 2>&1 || [ -s "$HOME/.nvm/nvm.sh" ]; then
    echo "==> Current Node $(node -v) is too old. Switching to Node 22 via nvm..."
    # shellcheck source=/dev/null
    [ -s "$HOME/.nvm/nvm.sh" ] && . "$HOME/.nvm/nvm.sh"
    nvm use 22 >/dev/null 2>&1 || nvm use 22
    echo "==> Now using Node $(node -v)"
  else
    echo "ERROR: Node $(node -v) is too old (need >= 22.13). Install nvm or upgrade Node.js." >&2
    exit 1
  fi
fi

# --- Clone or update the docs site into the cache dir (on $DOCS_REF) ---
# The cache is disposable: hard-reset to the fetched ref so a stale checkout or
# locally-modified tracked files (e.g. .sync-lock.json) can't leave it behind.
if [ -d "$CACHE_DIR/.git" ]; then
  echo "==> Updating cached docs site in $CACHE_DIR ($DOCS_REF)"
  git -C "$CACHE_DIR" fetch --depth 1 origin "$DOCS_REF"
  git -C "$CACHE_DIR" reset --hard FETCH_HEAD
else
  echo "==> Cloning docs site into $CACHE_DIR ($DOCS_REF)"
  git clone --depth 1 --branch "$DOCS_REF" "$DOCS_REPO" "$CACHE_DIR"
fi

cd "$CACHE_DIR"

PKG="npm"; RUN="npm run"
if command -v pnpm >/dev/null 2>&1; then PKG="pnpm"; RUN="pnpm"; fi
$PKG install

case "$CMD" in
  dev)
    # Resolve the gears-rust docs source so the full site renders. Prefer a sibling
    # checkout (picks up local, uncommitted edits); else shallow-clone into a
    # gitignored cache. Override with GEARS_RUST_PATH to point at any checkout.
    GEARS_RUST_REPO="${GEARS_RUST_REPO:-https://github.com/constructorfabric/gears-rust.git}"
    if [ -z "${GEARS_RUST_PATH:-}" ]; then
      if [ -d "$REPO_ROOT/../gears-rust/docs/web-docs" ]; then
        GEARS_RUST_PATH="$(cd "$REPO_ROOT/../gears-rust" && pwd)"
      else
        GR="${GEARS_DOCS_SOURCES:-$REPO_ROOT/.web-docs-sources}/gears-rust"
        if [ -d "$GR/.git" ]; then
          echo "==> Updating cached gears-rust source in $GR"
          git -C "$GR" pull --ff-only || echo "WARNING: could not fast-forward cached gears-rust; using existing checkout." >&2
        else
          echo "==> Cloning gears-rust source into $GR"
          git clone --depth 1 "$GEARS_RUST_REPO" "$GR"
        fi
        GEARS_RUST_PATH="$GR"
      fi
    fi
    echo "==> gears-rust docs source: $GEARS_RUST_PATH"
    echo "==> Starting docs site at http://localhost:4321 (CLI from $REPO_ROOT/docs/web-docs)"
    CARGO_GEARS_PATH="$REPO_ROOT" GEARS_RUST_PATH="$GEARS_RUST_PATH" BASE=/ $RUN dev
    ;;
  lint)
    # Lint only this repo's docs. Point gears-rust at a non-existent path so the
    # sync skips it (otherwise it defaults to ../gears-rust, the sibling checkout,
    # and we'd lint both repos). Then run the site's markdownlint config.
    echo "==> Linting cargo-gears docs with the site's markdownlint config"
    CARGO_GEARS_PATH="$REPO_ROOT" GEARS_RUST_PATH="$CACHE_DIR/.no-gears-rust-source" $RUN sync
    $RUN lint:md
    ;;
  *)
    echo "ERROR: unknown command '$CMD' (expected: dev | lint)" >&2
    exit 1
    ;;
esac
