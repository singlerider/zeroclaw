#!/usr/bin/env bash
# Sync per-locale .po files with the English source.
#
# Pipeline:
#   1. mdbook-xgettext  → docs/book/po/messages.pot   (extract from English)
#   2. msgmerge         → for each locale, merge new/changed msgids into .po
#                         (changed entries marked fuzzy; new entries left empty)
#   3. fill-translations.py → AI-fills delta only (fuzzy + empty entries)
#                              (requires ANTHROPIC_API_KEY; skipped if no delta)
#
# Idempotent — re-running against unchanged source is a no-op (no AI calls).
# Works identically locally and in CI.
#
# Adding a new locale: see docs/book/src/developing/building-docs.md

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BOOK_DIR="$REPO_ROOT/docs/book"
PO_DIR="$BOOK_DIR/po"
POT_FILE="$PO_DIR/messages.pot"
LOCALES="${LOCALES:-en ja}"

mkdir -p "$PO_DIR"

echo "==> Extracting English msgids → $POT_FILE"
(cd "$BOOK_DIR" && MDBOOK_OUTPUT__XGETTEXT__POT_FILE="messages.pot" mdbook build -d po-extract >/dev/null)
# mdbook-xgettext (via the gettext preprocessor in xgettext mode) writes
# messages.pot under the configured output dir; normalize the location.
if [[ -f "$BOOK_DIR/po-extract/xgettext/messages.pot" ]]; then
    mv "$BOOK_DIR/po-extract/xgettext/messages.pot" "$POT_FILE"
fi
rm -rf "$BOOK_DIR/po-extract"

for locale in $LOCALES; do
    [[ "$locale" == "en" ]] && continue   # English is the source

    po_file="$PO_DIR/$locale.po"

    if [[ ! -f "$po_file" ]]; then
        echo "==> $locale: bootstrapping new .po from template"
        msginit --no-translator --locale="$locale" --input="$POT_FILE" --output="$po_file"
    else
        echo "==> $locale: msgmerge (mark new/changed entries)"
        msgmerge --update --backup=none --no-fuzzy-matching "$po_file" "$POT_FILE"
    fi

    delta=$(LANG=C msgfmt --statistics "$po_file" -o /dev/null 2>&1 \
        | grep -oP '\d+(?= (untranslated|fuzzy))' \
        | paste -sd+ \
        | bc 2>/dev/null || echo 0)
    if [[ "$delta" -gt 0 ]]; then
        if [[ -n "${ANTHROPIC_API_KEY:-}" ]]; then
            echo "==> $locale: AI-filling $delta entries"
            python3 "$REPO_ROOT/scripts/fill-translations.py" --po "$po_file" --locale "$locale"
        else
            echo "==> $locale: $delta entries need translation (set ANTHROPIC_API_KEY to auto-fill)"
        fi
    else
        echo "==> $locale: up to date, skipping AI step"
    fi
done

echo "==> Done. Run 'just docs-translate-stats' to review coverage."
