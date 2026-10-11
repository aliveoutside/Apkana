# Apkana

Desktop GUI built with **egui/eframe** for common Android APK workflows.

## Features

- Decode and rebuild APKs using apktool
- Sign APKs with apksigner, including debug keystore fallback
- Merge split packages (APKS/XAPK/APKM -> APK)
- Install APKs with adb
- Optional build pipeline: zipalign → sign → install
- Resizable output log panel, clipboard copy, and editable external tool paths

## Runtime requirements

- Java runtime
- `apktool`
- `apksigner`
- `zipalign`
- `adb`

## Local development

```bash
cargo check
cargo test
cargo run
```

## Release build

```bash
cargo build --release
```

Portable configuration from `config.toml` next to the binary is supported.
