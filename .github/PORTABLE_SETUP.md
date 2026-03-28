Apkana portable setup

This app is a GUI wrapper and expects external Android tools.

Required tools:
- Java runtime (for apktool/apksigner)
- apktool.jar
- apksigner
- zipalign
- adb

Portable mode:
- If `config.toml` exists next to `apkana` executable, the app loads it.
- This is the easiest way to share one self-contained folder with tool paths.
