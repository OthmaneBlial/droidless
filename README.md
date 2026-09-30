<div align="center">

<img src="docs/assets/droidless-banner.svg" alt="DROIDLESS — Run Android apps without Android. Experimental Rust runtime." width="100%">

<br>

[![Release](https://img.shields.io/github/v/release/OthmaneBlial/droidless?style=for-the-badge&color=d9ff63&labelColor=171c21)](https://github.com/OthmaneBlial/droidless/releases/latest)
[![Rust](https://img.shields.io/badge/Rust-1.95-ff987d?style=for-the-badge&logo=rust&labelColor=171c21)](rust-toolchain.toml)
[![License](https://img.shields.io/badge/license-Apache%202.0-d9ff63?style=for-the-badge&labelColor=171c21)](LICENSE)
[![Checks](https://img.shields.io/badge/checks-local%20only-fffdf7?style=for-the-badge&labelColor=171c21)](tools/ci.sh)

**[🌐 Explore the website](https://othmaneblial.github.io/droidless/)** ·
**[📖 Open the runtime notebook](https://othmaneblial.github.io/droidless/docs.html)** ·
**[📦 Grab v0.1.0](https://github.com/OthmaneBlial/droidless/releases/tag/v0.1.0)**

</div>

## 👋 Meet the little runtime with a big idea

Take an Android APK. Execute its own bytecode. Give it a native desktop window.

**DROIDLESS is an experimental Android compatibility runtime, written in Rust.**
It owns the DEX interpreter, managed heap, Android API bridge, resource system
and View model. A small AppKit bridge supplies the macOS controls.

No Android Emulator. No Android VM. No ART or borrowed Dalvik engine.
No browser renderer or hidden Android installation. Android SDK tools compile
our authored test APKs only; they do not participate in execution.

> 🧭 A working slice of Android, one real APK at a time. Compatibility is narrow;
> unsupported methods and opcodes report errors.

## 🧮 The calculator has left the phone

**An unmodified third-party calculator is interactive on macOS ARM64.**
[KasCalc 1.0](https://github.com/KasRoudra/simplecalculator/releases/tag/v1.0)
runs its original calculation and click-listener DEX bytecode through DROIDLESS.

| Native clicks | Actual display |
|---|---|
| `7 → + → 5 → =` | **12.0** |
| `8 → × → 8 → =` | **64.0** |
| `√64` | **8.0** |

<div align="center">
<img src="docs/assets/kascalc-native.png" alt="Actual KasCalc APK in a native macOS window, displaying 12.0 after clicking 7 plus 5 equals" width="290">
<p><sub>Real window. Real APK callbacks. Actual screenshot.</sub></p>
</div>

The upstream APK is SHA-256 checked, never rebuilt, patched or repackaged, and
is not redistributed here. Ten additional headless scenarios check its View
state. Menus, gestures, complete visual fidelity and every numeric edge case
remain unverified. [See the evidence](docs/verification.md).

## 🚀 Give it a spin

Build with **Rust 1.95.0** and Apple's **Command Line Tools** on macOS:

```sh
git clone https://github.com/OthmaneBlial/droidless.git
cd droidless
cargo build --release --locked
sh tools/fetch-kascalc.sh
# Hello, desktop calculator 👋
target/release/droidless artifacts/apks/KasCalc.apk
```

The [v0.1.0 release](https://github.com/OthmaneBlial/droidless/releases/tag/v0.1.0)
includes a locally tested macOS ARM64 CLI archive and SHA-256 checksum. The binary
has no Apple developer signature or notarization. Linux has a headless code path;
its build and native UI have not been verified.

<details>
<summary><strong>🔎 Prefer the terminal? Inspect, replay and trace.</strong></summary>

```sh
target/release/droidless inspect app.apk
target/release/droidless manifest app.apk
target/release/droidless dex app.apk
target/release/droidless classes app.apk
target/release/droidless methods app.apk
target/release/droidless resources app.apk

# Original APK listeners update the emitted JSON View tree
target/release/droidless run --headless artifacts/apks/KasCalc.apk \
  --click 7 --click + --click 5 --click = --stats

# Current source: authored multi-screen navigation fixture
target/release/droidless run --headless fixtures/generated/intents.apk \
  --click "Open detail" --back
```

Use `--trace-bytecode`, `--trace-methods`, `--trace-framework` or
`--trace-lifecycle` to follow execution. `--stats` / `--heap-stats` report real
counters and timings. `--click`, `--key`, `--back`, `--headless` and `inspect-ui`
support repeatable experiments. **Escape delivers Back** in the native window.
Current source adds `--input TEXT` for the first visible EditText, per-package
preferences by default, `--data-dir APPS_ROOT` and `--ephemeral` for memory-only runs.

</details>

## ⚙️ Bytecode in. Native views out.

<img src="docs/assets/runtime-path.svg" alt="APK → Rust DEX interpreter → Android API bridge → native AppKit controls; input returns to APK callbacks" width="100%">

| Crate | Its job |
|---|---|
| `droidless-formats` | Bounded APK, binary XML, DEX and compiled-resource parsers |
| `droidless-runtime` | Register interpreter, objects, GC, lifecycle, framework APIs and Views |
| `droidless` | CLI, native macOS rendering and input bridge |

[Architecture & decisions](docs/architecture.md) · [DEX VM](docs/dex-vm.md) ·
[Framework & UI](docs/framework.md)

## 🧰 What's in the toolbox?

| Capability | Proven scope |
|---|---|
| 📦 APK / manifest / resources | Real APKs and malformed-input checks; default resource configuration |
| 🧠 DEX 035–040 | Headers, digests, IDs, classes, code and try handlers; annotations/debug partial |
| ⚡ Own register interpreter | Arithmetic, wide values, branches, arrays, fields, dispatch and Java throw/catch |
| 🧹 Managed objects | Inheritance, strings, sticky class initialization and handle-based mark/sweep GC |
| 🪟 Native widgets | TextView, Button, EditText, LinearLayout and FrameLayout; approximate layout/style |
| 🖱️ Input | Native calculator mouse callbacks, Counter text/key callbacks and clean close |
| 🧭 Activity navigation | Explicit same-APK Intents, typed Bundle extras, preserved Back stack, finish and lifecycle — current source |
| 📓 Persistent preferences | Typed SharedPreferences, isolated package data, native authored-note save/restart/clear — current source |
| 🛠️ Next up | General app files, collections, lists, images and timers; a substantial unmodified notes/todo APK |

The published **v0.1.0 archive predates navigation and persistence**. Current source capability
is documented separately. AndroidX, Compose, JNI, networking, Linux native UI
and games remain future compatibility work. [Exact limits](docs/compatibility.md).

## 🧪 Tiny lab. Real bytecode.

| APK | Origin | What we observed |
|---|---|---|
| KasCalc 1.0 | [Independent release](https://github.com/KasRoudra/simplecalculator/releases/tag/v1.0) | Native calculations; ten headless arithmetic/input cases |
| SmallestAPK | [Independent sample](https://github.com/krossovochkin/SmallestAPK) | Original signed APK executes its Activity and creates the expected TextView |
| Counter | [Authored Java/XML fixture](examples/counter/MainActivity.java) | Native text/key callbacks, resources and VM conformance |
| Intents | [Authored Java/XML fixture](examples/intents/MainActivity.java) | Native screen transitions, retained text/title, Back override, extras and lifecycle/GC checks |
| Preferences | [Authored Java/XML fixture](examples/preferences/MainActivity.java) | Native UTF-8 paste, save/restart/clear, typed values and package/path isolation |
| Notepad 1.0.0 | [Independent release](https://github.com/MohMah/android-notepad/releases/tag/v1.0.0) | Parsed; application startup stops at unsupported HashSet. No UI or working-notes claim |

Authored fixtures test implementation; they do not establish arbitrary APK
compatibility. [Evidence catalog](compatibility/catalog.json).

```sh
# CI lives on your machine. GitHub Actions stays disabled.
sh tools/ci.sh
python3 tools/compatibility.py
```

Local checks cover formatting, builds, tests, Clippy and 4,096 seeded parser
mutations. Normal tests require neither an Android SDK nor an emulator.

<details>
<summary><strong>🔧 Rebuild fixtures or create a development app bundle</strong></summary>

```sh
python3 tools/build-fixtures.py --sdk "$ANDROID_SDK_ROOT"
sh tools/macos-app.sh
```

The bundle is unsigned and intended for local development.

</details>

## 🗺️ Next checkpoint: 50%

The **20% milestone** is demonstrated: an unmodified APK renders, accepts clicks,
executes its own logic and updates native UI without Android. This is a milestone
label, not a measurement of Android API coverage.

The next ambition is **50%**: move beyond calculators toward useful everyday
apps, with multiple screens, persistent data, lists, images and scheduled callbacks.
That target remains ahead of us. Every step needs executable evidence and a
clear compatibility boundary. [Follow the roadmap](docs/roadmap.md).

## 🔐 A small but important boundary

This prototype is not a security sandbox. Parser/VM limits do not replace OS
isolation. Current source grants only isolated SharedPreferences I/O; general files,
networking, native-library and process APIs are unavailable. Use trusted APKs.
[Storage behavior and limits](docs/storage.md).
[Security boundaries](docs/security.md).

---

<div align="center">

**Built in Rust. Powered by original APK bytecode. A little stubborn by design. 🤖**

DROIDLESS is Apache-2.0. Third-party artifacts retain their own licenses.

[Website](https://othmaneblial.github.io/droidless/) · [Documentation](https://othmaneblial.github.io/droidless/docs.html) · [Source](https://github.com/OthmaneBlial/droidless)

</div>
