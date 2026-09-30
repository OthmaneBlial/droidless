# DROIDLESS

**Run Android apps without Android.**

An experimental Android compatibility runtime written in Rust. The project owns
its APK/binary XML/DEX/resource parsers and will execute application bytecode in
its own register machine, mapping Android framework calls to native desktop UI.

Current milestone: APK inspection. Execution and desktop rendering are being
implemented. Parsing an APK does **not** mean that it can run.

```sh
cargo build --release
target/release/droidless inspect fixtures/third-party/smallest.apk
target/release/droidless manifest fixtures/third-party/smallest.apk
target/release/droidless dex fixtures/third-party/smallest.apk
sh tools/ci.sh
```

CI runs **locally only**, by request. No GitHub Actions workflow is installed.
Runtime builds require Rust; Android SDK tools are only for compiling test APKs.

License: Apache-2.0. Third-party fixtures retain their original licenses.
