# Binary Size & Build Time Tracking Schema

## Data files

- [`binary-size-tracking.csv`](binary-size-tracking.csv) — Binary sizes, section sizes, dep counts, build times
- [`binary-size-tracking-pre-extraction.csv`](binary-size-tracking-pre-extraction.csv) — Archived pre-workspace-extraction measurements
- [`incremental-compile-times.csv`](incremental-compile-times.csv) — Incremental rebuild times per workspace crate

## binary-size-tracking.csv

### Columns

| Column | Type | Description |
|---|---|---|
| `test_id` | string | Unique identifier: `{profile}-{linker}-{features_short}` |
| `date` | `YYYY-MM-DD` | Date of measurement |
| `commit` | string | Short git commit hash |
| `profile` | enum | `dev`, `release`, `release-fast`, `ci` |
| `linker` | enum | `gnu-ld`, `mold`, `lld` |
| `codegen_backend` | enum | `llvm`, `cranelift` |
| `features` | string | Feature flags used (human-readable) |
| `binary_bytes` | int | `stat --format='%s'` |
| `binary_mb` | float | Computed |
| `text_bytes` | int | `.text` section from `readelf -SW` |
| `rodata_bytes` | int | `.rodata` section |
| `eh_frame_bytes` | int | `.eh_frame` section |
| `data_rel_ro_bytes` | int | `.data.rel.ro` section |
| `dep_tree_entries` | int | `cargo tree --edges=normal \| wc -l` |
| `duplicate_pairs` | int | `cargo tree --duplicates \| grep "^[a-z]" \| wc -l` |
| `build_time_secs` | int | Wall-clock seconds |
| `notes` | string | Free-text |

### Ad-hoc re-run

1. Find the `test_id` in the CSV
2. Reconstruct: `cargo build --profile {profile} [--features ...] [--no-default-features]`
3. For linker override: `CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER="clang" CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C link-arg=-fuse-ld={linker}"`
4. Measure: `stat`, `readelf`, `cargo tree`
5. Replace the row in the CSV

## incremental-compile-times.csv

### Columns

| Column | Type | Description |
|---|---|---|
| `test_id` | string | `{profile}-{touched_file_short}` |
| `date` | `YYYY-MM-DD` | Date |
| `commit` | string | Short git commit hash |
| `profile` | enum | `dev`, `release`, `ci` |
| `touched_file` | string | Full path of file touched |
| `crates_recompiled` | int | Count of `Compiling` lines in cargo output |
| `crate_names` | string | Semicolon-separated list of recompiled crate names |
| `run1_secs` | float | First timed run (seconds) |
| `run2_secs` | float | Second timed run |
| `run3_secs` | float | Third timed run |
| `median_secs` | float | Median of 3 runs |
| `notes` | string | Free-text |

### Methodology

1. Clean build to populate cache (once per profile)
2. Warmup run: `touch <file> && cargo build [--profile X]` — discard
3. Three timed runs: `touch <file> && time cargo build [--profile X]`
4. Record wall-clock time and which crates recompiled
5. Take median of 3 runs

## Measurement commands

```bash
# Binary size
cargo build --release && stat --format='%s' target/release/zeroclaw

# Section sizes
readelf -SW target/release/zeroclaw | grep -E "\.text |\.rodata |\.eh_frame |\.data\.rel"

# Dependency counts
cargo tree --edges=normal | wc -l
cargo tree --duplicates | grep "^[a-z]" | wc -l

# Per-crate size breakdown
cargo bloat --release --crates -n 30

# Strip .eh_frame (post-build)
objcopy --remove-section=.eh_frame --remove-section=.eh_frame_hdr target/release/zeroclaw

# Linker override (mold)
CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER="clang" \
CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C link-arg=-fuse-ld=mold" \
cargo build --release
```

## Charting

The CSVs are importable into Google Sheets, LibreOffice Calc, or any data tool.

- **Bar chart:** `binary_mb` grouped by `test_id` — feature gate impact
- **Stacked bar:** section sizes per build config
- **Grouped bar:** `median_secs` by `touched_file` grouped by `profile` — incremental timing comparison
- **Scatter:** `dep_tree_entries` vs `binary_mb` — dep count vs size correlation
