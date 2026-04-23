# Docs & Translations

ZeroClaw has two independent translation layers:

| Layer | Format | What it covers |
|---|---|---|
| **App strings** | Mozilla Fluent (`.ftl`) | CLI help text, command descriptions, runtime messages |
| **Docs** | gettext (`.po`) | Everything in this mdBook |

They are filled separately and stored separately. Both use Claude as the translation backend.

## Building the docs locally

{{#include ../developing/building-docs.md}}

## Filling app strings (Fluent)

App strings live in `crates/zeroclaw-runtime/locales/`. English is the source of truth and is embedded at compile time. Non-English locales are loaded from `~/.zeroclaw/workspace/locales/` at runtime.

Configure a provider in `config.toml` once:

```toml
[providers.models.ollama]
name = "ollama"
base_url = "http://localhost:11434"
model = "llama3.2"
```

Then:

```bash
cargo fluent stats                                      # coverage per locale
cargo fluent check                                      # validate .ftl syntax
cargo fluent fill --locale ja --provider ollama         # fill missing keys
cargo fluent fill --locale ja --provider ollama --force # retranslate everything
cargo fluent scan                                       # find stale or missing keys vs Rust source
```

After filling, copy the updated `.ftl` file to your workspace and rebuild the binary to pick up the changes:

```bash
cp crates/zeroclaw-runtime/locales/ja/cli.ftl ~/.zeroclaw/workspace/locales/ja/cli.ftl
```

## Filling doc translations (gettext)

Doc translations live in `docs/book/po/`. `cargo mdbook sync` runs extract → merge → AI-fill in one step. Without `--provider`, sync still runs extract + merge and reports how many strings need translation — partial translations fall back to English at render time.

```bash
cargo mdbook sync --provider ollama
cargo mdbook sync --provider ollama --force
```

## Adding a new locale

1. Add the locale code to `crates/zeroclaw-runtime/locales/` — create the directory and run `cargo fluent fill --locale <code>`.

2. Add the locale to `xtask/src/util.rs` → `locales()`:
   ```rust
   &["en", "ja", "<code>"]
   ```

3. Bootstrap the `.po` file for docs:
   ```bash
   msginit --no-translator --locale=<code> \
     --input=docs/book/po/messages.pot \
     --output=docs/book/po/<code>.po
   ```
   Then run `cargo mdbook sync --locale <code>` to fill it.

4. Register the locale in the lang switcher — `docs/book/theme/lang-switcher.js`:
   ```js
   { code: "<code>", label: "Language Name" },
   ```

5. Add the locale to the deploy workflow — `.github/workflows/docs-deploy.yml`:
   ```yaml
   LOCALES: en ja <code>
   ```
