# Reproducible local measurements

```sh
sh tools/fetch-kascalc.sh
cargo run --release --locked -p droidless-runtime --example benchmark -- artifacts/apks/KasCalc.apk
```

Eleven samples in one warm process report medians for full APK decode
(ZIP/manifest/DEX/resources), headless Activity launch plus View layout, and
instruction throughput of a compiled Java integer loop. These are real timings
on the invoking host, not predicted numbers or Android/emulator comparisons.
Native first-frame/fps/GC pauses/RSS are not measured by this benchmark.
The archived initial measurement is `macos-arm64-initial.json`; paths and host
architecture are reported with the scope. Re-run rather than assuming old numbers
apply to another host/build/application.
