# Overnight Research TODO

## Methodology verification — DONE
- [x] Confirm all CSV columns are populated for existing measurements
- [x] Verify section size parsing is consistent (hex → decimal)
- [x] Check that build times are from clean or incremental (document which)

## Remaining feature-gate measurements — DONE
- [x] Per-gate section sizes (text/rodata/eh_frame/data_rel_ro) for each of the 4 gates individually
- [x] Dep tree counts for without-email and without-tui (missing duplicate pair counts)
- [ ] cargo-bloat per-crate WITHOUT all 4 gates (compare what disappears) — DEFERRED (10min build per bloat run)

## .eh_frame investigation (singlerider/zeroclaw#5) — DONE
- [x] Test `-C force-unwind-tables=no` on nightly release build — NO EFFECT
- [x] Test `-Wl,--gc-sections` linker flag on stable release build — NO EFFECT (already default)
- [x] Test `-Wl,--strip-all` + force-unwind-tables=no — NO EFFECT
- [x] Test `objcopy --remove-section=.eh_frame` — **WORKS: -2.0 MB (10.1%)**
- [x] Verify runtime: --version, --help, doctor, error handling all pass
- [x] singlerider/zeroclaw#5 updated

## schemars / serde Config bloat investigation — DONE
- [x] Count Config struct fields — ~1,159 pub fields across all nested structs in schema.rs
- [x] Check if schemars can be made optional — YES, only used in `zeroclaw config schema` CLI command + 1 test
- [x] Estimated savings: ~457 KiB (.text) if schemars made optional (373 KiB crate + 84 KiB json_schema fn)
- Created singlerider/zeroclaw#8 for this research area

## Existing feature gates measurement (for comparison) — DONE
- [x] channel-nostr: +1.97 MB (+10.0%), +80 deps
- [x] whatsapp-web: +3.56 MB (+18.0%), +256 deps
- [x] channel-lark: +0.15 MB (+0.8%), +9 deps
- [x] hardware: +0.11 MB (+0.6%), +30 deps
- [x] plugins-wasm: +0.09 MB (+0.5%), +430 deps (compile-time cost, not binary)
- [x] probe: +0.00 MB (0.0%), +189 deps (compile-time cost only)
- [x] rag-pdf: +1.19 MB (+6.0%), +47 deps
- [x] browser-native: +0.27 MB (+1.4%), +84 deps

## regex audit — DONE
- [x] Find all regex usage sites in src/ — 100 sites across 15 files
- [x] Check if any can use simple string matching — NO, all are legitimate patterns
- [x] Document regex_automata + aho_corasick + regex_syntax combined cost (515 KiB from bloat) — unavoidable

## reqwest feature audit — DONE
- [x] Find all `reqwest::blocking` call sites — 15 sites in 4 files; needed for sync CLI/onboarding
- [x] Find all SOCKS proxy usage sites — config/schema.rs WebSocket proxy; needed for proxy users
- [x] Document whether blocking/socks could be gated — NOT RECOMMENDED; would break core functionality

## Update tracking — DONE
- [x] Update CSV with all new measurements (26 rows now)
- [x] Update singlerider/zeroclaw#5 with .eh_frame findings
- [x] Update zeroclaw-labs/zeroclaw#5272 measurement comment (consolidated all data into single comment)
- [x] Update zeroclaw-labs/zeroclaw#5272 methodology comment
- [x] Created singlerider/zeroclaw#8 for schemars optimization
- [x] Updated zeroclaw-labs/zeroclaw#5272 issue body with corrected baseline

## Completed research summary

### Key findings
1. **objcopy .eh_frame removal: -2.0 MB (10.1%)** — zero code changes, post-build script
2. **4 new feature gates combined: -1.31 MB (6.6%)** — when all disabled
3. **schemars optional: ~457 KiB estimated** — medium effort (167 derive sites)
4. **channel-matrix is the heavyweight: +10.72 MB** — validates existing gate
5. **whatsapp-web: +3.56 MB, channel-nostr: +1.97 MB** — also correctly gated
6. **Linker choice (mold vs GNU ld): negligible** — codegen dominates, not linking
7. **Cranelift: 14% faster clean debug builds** — but no incremental benefit
8. **regex (515 KiB): unavoidable** — 100 legitimate usage sites
9. **reqwest blocking/socks: cannot gate** — needed for CLI/onboarding + proxy
10. **StageX: blocked** — ships Rust 1.82, project requires 1.85+

### Theoretical minimum binary
- Start: 19.80 MB (default)
- objcopy .eh_frame: -2.0 MB → 17.80 MB
- Disable all 4 new gates: -1.31 MB → 16.49 MB
- Schemars optional: -0.45 MB → 16.04 MB
- **Theoretical floor: ~16 MB** (without removing any always-compiled functionality)
