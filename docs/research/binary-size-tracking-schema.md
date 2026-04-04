# Binary Size & Build Time Tracking Schema

Data file: [`binary-size-tracking.csv`](binary-size-tracking.csv)

## Columns

| Column | Type | Description |
|---|---|---|
| `date` | `YYYY-MM-DD` | Date of measurement |
| `branch` | string | Git branch name |
| `profile` | enum | Cargo profile: `dev`, `release`, `release-fast`, `ci`, `dist` |
| `features` | string | Feature flags used (human-readable summary) |
| `linker` | enum | `gnu-ld`, `mold`, `lld`, `wild` |
| `codegen_backend` | enum | `llvm`, `cranelift` |
| `binary_bytes` | int | Binary size in bytes (`stat --format='%s'`) |
| `binary_mb` | float | Binary size in MB (computed) |
| `text_bytes` | int | `.text` section size from `readelf -SW` |
| `rodata_bytes` | int | `.rodata` section size |
| `eh_frame_bytes` | int | `.eh_frame` section size |
| `data_rel_ro_bytes` | int | `.data.rel.ro` section size |
| `dep_tree_entries` | int | `cargo tree --edges=normal \| wc -l` |
| `duplicate_pairs` | int | `cargo tree --duplicates \| grep "^[a-z]" \| wc -l` |
| `build_time_secs` | int | Wall-clock build time in seconds |
| `notes` | string | Free-text context |

## How to add a measurement

```bash
# 1. Record metadata
DATE=$(date +%Y-%m-%d)
BRANCH=$(git branch --show-current)
PROFILE=release
FEATURES="description of features"
LINKER=gnu-ld
BACKEND=llvm

# 2. Build and measure
time cargo build --release [--features ...] [--no-default-features]
BINARY_BYTES=$(stat --format='%s' target/release/zeroclaw)

# 3. Section sizes (parse hex from readelf)
readelf -SW target/release/zeroclaw | grep -E "\.text |\.rodata |\.eh_frame |\.data\.rel"

# 4. Dependency counts
DEP_ENTRIES=$(cargo tree --edges=normal [--features ...] | wc -l)
DUP_PAIRS=$(cargo tree --duplicates [--features ...] | grep "^[a-z]" | wc -l)

# 5. Append to CSV
echo "$DATE,$BRANCH,$PROFILE,$FEATURES,$LINKER,$BACKEND,$BINARY_BYTES,..." >> docs/research/binary-size-tracking.csv
```

## Quick reference: hex to decimal for section sizes

```python
python3 -c "print(0xABCDEF)"  # paste hex value from readelf
```

## Charting

The CSV is importable into Google Sheets, LibreOffice Calc, or any data tool. Suggested charts:

- **Bar chart:** `binary_mb` grouped by `features` — shows impact of each feature gate
- **Stacked bar:** section sizes (`text`, `rodata`, `eh_frame`, `data_rel_ro`) per build config
- **Line chart:** `binary_mb` over `date` — tracks size drift over time
- **Scatter:** `dep_tree_entries` vs `binary_mb` — correlation between dep count and size
