# Findings

## Before

- Monolithic crate: **289,000 LOC**, single compilation unit
- Dev incremental rebuild: **20.8s** (any file change)
- Release incremental: **338.8s**
- CI incremental: **188.3s**
- Default binary: **22.2 MB**
- Dev clean build with gnu-ld: **210s**

## After

- 8 workspace crates, root reduced to **106,000 LOC**
- Dev incremental rebuild: **10.0s** (-52%)
- Release incremental: **234.4s** (-31%)
- CI incremental: **105.6s** (-44%)
- Minimum binary (feature gates + .eh_frame strip): **18.8 MB**
- Dev clean build with mold: **120s** (-43%)

## Do this now

1. **Add to release build script:**
   ```bash
   objcopy --remove-section=.eh_frame --remove-section=.eh_frame_hdr target/release/zeroclaw
   ```
   Saves 2.2 MB. Binary verified working.

2. **Add to `.cargo/config.toml`:**
   ```toml
   [target.x86_64-unknown-linux-gnu]
   linker = "clang"
   rustflags = ["-C", "link-arg=-fuse-ld=mold"]
   ```
   Dev builds link 43% faster.

3. **Adopt the workspace split.** Every developer rebuild is 31-64% faster. It compounds.
