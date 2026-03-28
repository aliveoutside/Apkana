# AGENTS.md - Apkana

## Purpose

`Apkana` is a Rust desktop app (`iced 0.14`) for common APK workflows.

Main tabs:

1. Decode + Build
2. Sign
3. Merge (APKS/XAPK/APKM -> APK, native Rust merge path)
4. Install (`adb install -r`)

## Fast Verify

Run from repo root:

```bash
cargo check
cargo test
```

Baseline today: 9 unit tests (`cargo test -- --list`).

## Release Flow

- Tag-driven GitHub release workflow: `.github/workflows/release.yml`
- Trigger: push tag `v*`
- Output assets:
  - `apkana-linux-x86_64.tar.gz`
  - `apkana-windows-x86_64.zip`
  - `SHA256SUMS`
- Release body source: `.github/RELEASE_NOTES.md`

## Runtime Tooling Model

The app runs external tools via configured paths (`src/config.rs`):

- Java runtime
- `apktool.jar`
- `apksigner`
- `zipalign`
- `adb`

Portable mode is supported: if `config.toml` exists next to the executable, it is preferred over user config.

## Architecture Map

- `src/app.rs`
  - `ApkanaApp` state and top-level `Message` routing
- `src/app/handlers.rs`
  - Per-tab update handlers (`update_decode`, `update_build`, `update_sign`, `update_merge`, `update_install`)
- `src/app/pipeline.rs`
  - Async completion handlers and shared task starters
- `src/app/validation.rs`
  - Input validation for each workflow
- `src/app/view.rs`
  - Main tab routing
- `src/ui/*_tab.rs`
  - Tab-local state and rendering
- `src/tools/*`
  - Wrappers for external tools + merger
- `src/tools/merger/*`
  - Native split-package merge engine

## Merger Guardrails

Key files:

- `src/tools/merger/mod.rs` (flow + `MergeOptions`)
- `src/tools/merger/archive.rs` (archive detection/extraction)
- `src/tools/merger/apk_merge.rs` (zip merge + entry rules)
- `src/tools/merger/manifest.rs` (manifest sanitization)
- `src/tools/merger/resources.rs` (ARSC merge)

Behavior that should be preserved:

- Remove only signature-related `META-INF` entries; keep non-signature files (for example `META-INF/services/*`).
- If ARSC parse/serialize fails, keep base `resources.arsc` and continue with warning instead of aborting merge.
- Detect/log OBB files in XAPK; do not extract them as APK content.

## Dependency Override

`Cargo.toml` patches `resand`:

```toml
[patch.crates-io]
resand = { git = "https://codeberg.org/goldsmith1433/resand.git", rev = "ac62f1ddb3817f3da358f0f398774e8173f86c8d" }
```

Do not change this revision without re-running merge-focused checks.

## Known Gaps

1. More real-world fixture validation is still needed for complex split sets.
2. There are no integration tests yet for full APKS/XAPK/APKM archives (only unit-level coverage).
