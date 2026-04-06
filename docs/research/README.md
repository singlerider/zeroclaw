# Binary Size & Compile Time Research

**Branch:** `issue/compiled-size-improvements`
**Upstream tracker:** [zeroclaw-labs/zeroclaw#5272](https://github.com/zeroclaw-labs/zeroclaw/issues/5272)
**Sub-issues:** [singlerider/zeroclaw#1-8](https://github.com/singlerider/zeroclaw/issues)

## Summary

Research and implementation to reduce the ZeroClaw compiled binary footprint and improve incremental compilation times. Started 2026-04-04.

**Results:**
- 5 workspace crates extracted (62,603 LOC out of root crate)
- 28 channel feature gates (all channels independently toggleable)
- schemars made optional (~457 KB savings)
- .eh_frame post-build strip saves 2.0 MB
- Root crate modules (config, memory, providers) replaced with thin re-exports

## Files in this directory

| File | Description |
|---|---|
| [`PROGRESS.md`](PROGRESS.md) | Detailed log of all work done, findings, and decisions |
| [`TODO.md`](TODO.md) | Task tracking with completion status and future work |
| [`binary-size-tracking.csv`](binary-size-tracking.csv) | 19 measurement rows across feature combinations, profiles, linkers |
| [`binary-size-tracking-schema.md`](binary-size-tracking-schema.md) | CSV column definitions, measurement commands, charting suggestions |
| [`cargo-bloat-crates.txt`](cargo-bloat-crates.txt) | Per-crate .text section breakdown (top 30 by size) |
| [`cargo-bloat-functions.txt`](cargo-bloat-functions.txt) | Per-function .text breakdown (top 40 by size) |

## Quick reference

### Workspace crates

| Crate | LOC | Content |
|---|---|---|
| `zeroclaw-types` | 594 | Channel/Tool traits, message types, Summarizer trait |
| `zeroclaw-infra` | 1,847 | Session backends, debounce, stall watchdog |
| `zeroclaw-config` | 18,262 | Config schema, secrets, proxy runtime, scattered types |
| `zeroclaw-memory` | 11,041 | SQLite/Qdrant, embeddings, consolidation, retrieval |
| `zeroclaw-providers` | 30,859 | All LLM providers, auth, multimodal processing |

### Measurement commands

```bash
# Binary size
cargo build --release && stat --format='%s' target/release/zeroclaw

# Section sizes
readelf -SW target/release/zeroclaw | grep -E "\.text |\.rodata |\.eh_frame"

# Dependency counts
cargo tree --edges=normal | wc -l
cargo tree --duplicates | grep "^[a-z]" | wc -l

# Per-crate size breakdown
cargo bloat --release --crates -n 30

# Strip .eh_frame (post-build)
objcopy --remove-section=.eh_frame --remove-section=.eh_frame_hdr target/release/zeroclaw
```

### Sub-issues

| # | Title |
|---|---|
| [singlerider/zeroclaw#1](https://github.com/singlerider/zeroclaw/issues/1) | Feature-gate channel-email |
| [singlerider/zeroclaw#2](https://github.com/singlerider/zeroclaw/issues/2) | Feature-gate channel-mqtt |
| [singlerider/zeroclaw#3](https://github.com/singlerider/zeroclaw/issues/3) | Feature-gate tui-onboarding |
| [singlerider/zeroclaw#4](https://github.com/singlerider/zeroclaw/issues/4) | Feature-gate channel-telegram |
| [singlerider/zeroclaw#5](https://github.com/singlerider/zeroclaw/issues/5) | Eliminate .eh_frame bloat |
| [singlerider/zeroclaw#6](https://github.com/singlerider/zeroclaw/issues/6) | Workspace restructuring |
| [singlerider/zeroclaw#7](https://github.com/singlerider/zeroclaw/issues/7) | StageX + linker/allocator exploration |
| [singlerider/zeroclaw#8](https://github.com/singlerider/zeroclaw/issues/8) | Make schemars optional |
