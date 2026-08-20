# Apkana

Desktop GUI for common Android APK workflows.

## What it does

- Decode and rebuild APKs
- Sign APKs
- Merge split packages (APKS/XAPK/APKM -> APK)
- Install APK 

## Screenshots
<img src=".github/screenshots/1.png" alt="Decode view" width="800">
<img src=".github/screenshots/2.png" alt="Merge view" width="800">
<img src=".github/screenshots/3.png" alt="Sign view" width="800">


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
```

## Build

```bash
cargo build --release
```
