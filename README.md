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

## Merge APK

Merge accepts **one compatible, complete split set** in `.apks`, `.xapk` or
`.apkm`. Base/feature/configuration identity, package/version and dependencies
come from binary APK manifests; ABI inventory and DPI/language qualifiers come
from the payload. File names do not determine module identity. Density and
language resources retain their original ARSC configurations; complete ABI
library sets remain in separate `lib/<abi>/` directories.

Feature manifest declarations, resource packages/string references, DEX and
assets are preserved. Different bytes at the same ZIP path, duplicate classes,
conflicting resource IDs/configurations, missing features, mismatched signing
certificates and ambiguous variants cause an error instead of silent data loss.

The result is staged, aligned, signed and checked using `apksigner verify`,
`zipalign -c` and `aapt2 dump` before replacing the destination. Failure leaves
an existing output untouched. Configure `apksigner`, `zipalign` and `aapt2` in
Settings. Merge uses the key configured in the Sign tab, or an existing
`~/.android/debug.keystore` with the standard Android debug credentials.
Re-signing changes the app's identity for updates and certificate-based APIs.

Currently unsupported archives fail with an explanation: multiple standalone
or split variants (extract a single compatible set first), isolated split class
loaders, incompatible application/SDK overrides, OBB expansion data, incomplete
multi-ABI library sets, styled ARSC string pools, compact resource entries and
ARSC overlay/staged-alias chunks. Library package mappings are preserved and
checked for conflicts. No `toc.pb` variant selector or Play on-demand delivery
emulation is provided. A static verification cannot prove arbitrary application
behavior; test your resulting APK on its target Android device.

### Integration checks

`tests/fixtures/feature.{apks,xapk,apkm}` contain real signed APK binaries built
from our own small MIT-licensed Android app, with deliberately misleading file
names. They exercise a feature Activity/Service/Receiver/Provider and permission,
multiple DEX files, a separate feature resource package, French and density
resources, assets, and an x86_64 JNI library. No third-party APK is redistributed.

```sh
# SDK build tools and Java/keytool must be on PATH.
cargo test --locked
cargo test --locked --test merge_archives -- --ignored
# A dedicated emulator/Waydroid target must be connected and ready.
python3 tests/android/run_on_android.py --serial 192.168.240.112:5555
```

The Android test installs only `dev.apkana.mergefixture`, launches it and requires
success markers for resources/JNI, Provider, Service and Receiver, plus a live
feature Activity. Fixture regeneration is documented in
`tests/android/generate_fixtures.py`; `ANDROID_JAR` and `D8_JAR` select SDK inputs.
