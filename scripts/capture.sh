#!/usr/bin/env bash
# Record the media of every capture story into the docs repository.
# A story is named after the page it captures for, and runs for each locale that has the page.
# Usage: capture.sh <docs-repo>
set -euo pipefail

DOCS="$(cd "$1" && pwd)"
# Locales of the docs and the editor language each is captured in
LOCALES="zh:zh_cn en:en_us"

cd "$(dirname "$0")/.."

# cargo provides the manifest directory the editor finds its assets by
run() { cargo run -q -p phichain --features capture -- "$@"; }

cargo build -q -p phichain --features capture
stories=$(run --capture-list 2>/dev/null)

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

for pair in $LOCALES; do
    locale="${pair%%:*}"
    language="${pair#*:}"
    for story in $stories; do
        page="$DOCS/src/content/docs/$locale/$story"
        if [ ! -e "$page.mdx" ] && [ ! -e "$page.md" ]; then
            continue
        fi
        echo "capturing $story in $language"
        log="$work/$story.$locale.log"
        if ! run --capture "$story" --capture-language "$language" --capture-out "$work/$locale" >"$log" 2>&1; then
            tail -n 30 "$log" >&2
            echo "capturing $story in $language failed" >&2
            exit 1
        fi
    done

    # replaced as a whole, so media of removed pages and stories go away too
    rm -rf "$DOCS/src/captures/$locale"
    if [ -d "$work/$locale" ]; then
        mkdir -p "$DOCS/src/captures"
        mv "$work/$locale" "$DOCS/src/captures/$locale"
    fi
done
