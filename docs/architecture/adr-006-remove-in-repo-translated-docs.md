# ADR-006: Remove In-Repo Translated Documentation

**Status:** Proposed

**Date:** 2026-04-16

**Issue:** [#5787](https://github.com/zeroclaw-labs/zeroclaw/issues/5787)

## Context

The repository contains 169 manually-maintained translated markdown files in
`docs/i18n/` across 31 locales, plus 31 `README.*.md` files at the root.
There is no CI staleness check, no PO-style source-change tracking, and no
community translation workflow.

Every English source edit silently invalidates an unknown number of
translations. Issue #5679 documents 30+ translated setup guides referencing
`install.sh` flags that were removed in #5666 — with no automated detection.
This is the expected steady state under manual maintenance, not an anomaly.

RFC #5576 §4 recommends removing in-repo translations entirely.

## Decision

### 1. Delete all in-repo translated documentation

Remove:
- `docs/i18n/` (169 files, ~2.2 MB)
- All `README.*.md` at root (31 files)
- Non-English hub files in `docs/`
- `docs/maintainers/i18n-coverage.md` and i18n index files
- The i18n parity requirement from `docs/contributing/docs-contract.md`

Tag a release before the deletion PR so the content is permanently archived
at that SHA.

### 2. Replace with on-demand LLM translation

A `zeroclaw docs --translate <path> --locale <lang>` CLI command uses the
user's configured LLM provider to translate any doc page on demand. This is
more accurate (always uses the latest source), requires zero maintenance, and
demonstrates ZeroClaw's own capability.

### 3. Wiki stays English only

The GitHub Wiki is not a translation platform. On-demand translation via the
CLI command is the path for non-English users.

### 4. Add a `TRANSLATIONS.md` stub

A root-level `TRANSLATIONS.md` directs users to `zeroclaw docs --translate`
for translated documentation.

## Consequences

### Positive

- Eliminates silent drift of translated docs against English source.
- Removes ~2.2 MB of maintenance-burdened content from the repo.
- Translated output is always current — generated from latest source on demand.
- Core contributors no longer bear translation maintenance.

### Negative

- Users without a configured LLM provider cannot get translated docs.
- On-demand translation quality depends on the provider model.
- Loss of community-contributed translations (archived at pre-deletion tag).

### Neutral

- The Wiki is unaffected — it remains English only.
- Existing English documentation is unchanged.

## References

- [RFC proposal](../proposals/mozilla-fluent-i18n.md)
- RFC #5576 §4 — Documentation Standards: remove in-repo translations
- Issue #5679 — translated setup guides reference removed flags
