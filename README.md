# An Electronic Manual for Keep Talking and Nobody Explodes

Build Desktop:

```
cargo build --features="desktop"
```

Add `--release` to build a release version.

Build Android:

```$env:ANDROID_NDK_HOME="path/to/ndk"
$env:ANDROID_HOME="path/to/sdk"

rustup target add aarch64-linux-android
cargo install cargo-ndk

cargo ndk --platform 21 -t arm64-v8a -o app/src/main/jniLibs/ build
./gradlew build
./gradlew installDebug
```

Add `--release` to the cargo ndk command to build a release version.
