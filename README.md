# Apkana

Desktop GUI for common Android APK workflows.

## What it does

- Decode and rebuild APKs
- Sign APKs
- Merge split packages (APKS/XAPK/APKM -> APK)
- Install APK with `adb install -r`

## Screenshots
[Decode view](.github/screenshots/1.png)
[Merge view](.github/screenshots/2.png)
[Sign view](.github/screenshots/3.png)

## Runtime requirements

`Apkana` calls external Android tools. Configure paths in app settings:

- Java runtime
- `apktool.jar`
- `apksigner`
- `zipalign`
- `adb`

Portable mode is supported: if `config.toml` is next to the executable, the app loads it from there.

## Local development

```bash
cargo check
cargo test
```

## Build

```bash
cargo build --release
```
