# First interactive milestone evidence

Host: Apple Silicon macOS. Date: 2026-09-30. Runtime: DROIDLESS Rust interpreter
plus AppKit controls. Native observations and headless tests are separated below.

## Independent artifact

KasCalc 1.0, `com.kasroudra.kascalc`, min SDK 8/target SDK 17,
[original upstream release](https://github.com/KasRoudra/simplecalculator/releases/tag/v1.0).
SHA-256: `6010d2f142cd8d0114ab627a44b50b5dc223b4d94ae5a235f836afbae211f505`.
The fetch helper verifies it. No DEX/layout/package modification or recompilation.
We do not bundle the GPL-3.0 calculator; its original source/license stay upstream.

## Actual native interaction

Computer-use automation inspected a real AppKit window and clicked the APK buttons:

| Event | Display observed in native accessibility state |
|---|---|
| 7 | 7 |
| + | 7.0 |
| 5 | 5 |
| = | 12.0 |
| AC, 8, x, 8, = | 64.0 |
| √ | 8.0 |

[Window screenshot](assets/kascalc-native.png) captures the real 12.0 state.
It is a screenshot, not a generated image; gradients/ripple rendering is partial.
`otool -L` showed AppKit/QuartzCore/Foundation/CoreFoundation/libSystem/libobjc;
process inspection found no emulator/QEMU/Waydroid/Anbox/app_process/dalvikvm.
Runtime source contains no subprocess fallback to Android.

Subsequent native Counter automation could not reattach (`cgWindowNotFound`),
although process sampling and native tracing confirmed its live event loop/window.
Counter native text input/close are therefore not independently verified yet.
Headless tests prove the runtime semantics, not macOS keyboard delivery.

## Automated checks

`sh tools/ci.sh` runs fmt/check/test/clippy/release locally. Checked-in unsigned
Counter APK is built from visible Java/XML using aapt/javac/D8; normal test runs
need neither SDK nor emulator. Tests cover malformed/truncated inputs, ZIP safety,
third-party Activity execution, arithmetic/wide values, fields/static initialization,
arrays/covariance, inherited interface dispatch, explicit/implicit exceptions,
17 fault paths and catch-all/finally, GC reachability,
lifecycle, DEX clicks and EditText copy.
Failed class-initialization tests verify wrapping versus Error propagation,
subsequent NoClassDefFoundError and cause retention across collection.
Local CI also runs 4,096 seeded APK/DEX/XML/resource mutations without panics.
This is deterministic smoke coverage, not a coverage-guided fuzz campaign.

`python3 tools/compatibility.py` verifies KasCalc's digest and checks ten actual
headless EditText results: arithmetic, decimals, roots, backspace, sign/reset.
KeyEvent replay additionally produced 2.0 for 7 minus 5 equals from its own listener.
Generated reports stay under ignored artifacts. No Android reference differential
run, Linux build/UI test, broad compatibility or external adoption is claimed.
No GitHub source CI was used.
