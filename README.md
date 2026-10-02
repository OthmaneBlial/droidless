<div align="center">

<img src="docs/assets/droidless-banner.svg" alt="DROIDLESS — Android apps. New desktop habitat. An experimental Rust runtime with native views." width="100%">

<br>

[![Release](https://img.shields.io/github/v/release/OthmaneBlial/droidless?style=for-the-badge&color=d9ff63&labelColor=171c21)](https://github.com/OthmaneBlial/droidless/releases/latest) [![Rust](https://img.shields.io/badge/Rust-1.95-83b8ff?style=for-the-badge&logo=rust&labelColor=171c21)](rust-toolchain.toml) [![License](https://img.shields.io/badge/license-Apache%202.0-d9ff63?style=for-the-badge&labelColor=171c21)](LICENSE) [![Checks](https://img.shields.io/badge/CI-local%20only-fffdf7?style=for-the-badge&labelColor=171c21)](tools/ci.sh)

**[📦 Download v0.3.0](https://github.com/OthmaneBlial/droidless/releases/tag/v0.3.0)** · **[🌐 Explore the website](https://othmaneblial.github.io/droidless/)** · **[📖 Open the notebook](https://othmaneblial.github.io/droidless/docs.html)**

**Original APK bytecode → our Rust engine → native desktop views.**

</div>

## 👋 A little runtime with a big idea

Give an Android app a new home on your desktop. Open a note, change its title and
body, save it, then restart: the original APK brings your edits back. ✍️

**DROIDLESS is an experimental Android compatibility runtime written in Rust.**
It owns the DEX interpreter, managed heap, resource system, Android API bridge
and View model. AppKit supplies native macOS controls; clicks and keyboard input
return to the APK's own callbacks.

The execution path runs without an Android emulator, ART, Dalvik or a hidden
Android installation. Android SDK tools are used only to compile authored test
fixtures. Compatibility is narrow, and unsupported APIs report explicit errors.

## ✨ What can it actually do?

| Real app | Verified native interaction | Verified headless replay |
|---|---|---|
| 📓 **[Notepad 1.0.0](https://github.com/MohMah/android-notepad/releases/tag/v1.0.0)** | Mouse selection, keyboard title/body edits, Back save and exact restoration in a fresh process | Two persistent SQLite notes; edit/restart; Delete/Undo; drawer; folder create/rename/delete; exact database backup/restore and fresh-launch recovery |
| 🖼️ **[SwpieView 1.3.2](https://f-droid.org/en/packages/org.voidptr.swpieview/)** | Folder chooser → three thumbnails → JPEG/PNG/WebP viewer → Escape back; taps hide/show controls | Original image swipes, first/last bounds, control visibility, Parcelable transfer and Back |
| 🧮 **[Simple Calculator 1.0](https://github.com/swiftugandan/Simple-Android-Calculator/tree/3ba860b281eba34f144e4e75115f0c0a06bced31)** | Original clicks produce `7 + 5 = 12`, `8 × 8 = 64` and `9 / 3 = 3` | Seven arithmetic and input scenarios |

Public APKs are pinned, SHA-256 checked and executed unchanged. They are fetched
from upstream and **never bundled in our release**. Native observations and
headless checks have separate evidence; authored fixtures establish engine
contracts, not arbitrary APK compatibility. [Read the verification record](docs/verification.md).

<div align="center">
<table align="center">
<tr>
<td align="center" width="50%" valign="top">
<img src="site/assets/notepad-preview.svg" alt="Illustrated headless Notepad View tree showing two saved notes." height="320"><br>
<strong>📓 Save. Reopen. Keep writing.</strong><br>
<sub>Illustrated View-tree preview, not a native capture.</sub>
</td>
<td align="center" width="50%" valign="top">
<img src="docs/assets/simple-calculator-native.png" alt="Actual public Simple Calculator APK in a native macOS window, showing 12." height="320"><br>
<strong>🧮 Original clicks. Original result.</strong><br>
<sub>Earlier native capture at a 192 × 400 viewport.</sub>
</td>
</tr>
</table>
</div>

<div align="center">

**📥 Open → ✍️ Edit → 💾 Save → 🗂️ Back up → 🔁 Restore → 🚀 Reopen**

</div>

<sub>Backup/restore is verified on an isolated data copy. The original APK calls `System.exit(0)` after restoring; that shutdown API remains unsupported, but a fresh DROIDLESS run confirms the exact database and notes were recovered.</sub>

## 🚀 Get your first APK running

**v0.3.0 is a macOS ARM64 preview.** The archive contains the CLI, five authored
fixtures, three hash-checking public APK fetch helpers and release provenance.
It has no Apple developer signature or notarization.

[Download the archive and SHA256SUMS](https://github.com/OthmaneBlial/droidless/releases/tag/v0.3.0),
then, from their download directory:

```sh
shasum -a 256 -c SHA256SUMS
tar -xzf droidless-0.3.0-macos-arm64.tar.gz
cd droidless-0.3.0-macos-arm64

# Your first guest callback 🤖
./droidless run --headless --ephemeral fixtures/counter.apk --click Increment
```

Try the **original public notes app**, using a dedicated local data directory:

```sh
sh tools/fetch-notepad.sh

# Notes → editor → save → return to the list
./droidless run --headless --size 390x844 --data-dir ./test-apps \
  --click "＋" --input "Hello, desktop" --input-at 1 "Made of Rust and curiosity." \
  --back artifacts/apks/notepad-v1.0.0.apk

# A fresh process restores your note
./droidless run --headless --size 390x844 --data-dir ./test-apps \
  artifacts/apks/notepad-v1.0.0.apk

# Open a native desktop window
./droidless run --size 390x720 --data-dir ./test-apps \
  artifacts/apks/notepad-v1.0.0.apk
```

**Escape delivers Back** in the native window. The package also includes
`fetch-simple-calculator.sh` and `fetch-swpieview.sh`.
[Release scope and known limits](docs/releases/0.3.0.md).

<details>
<summary><strong>🛠️ Prefer building from source?</strong></summary>

Use Rust **1.95.0** and Apple's Command Line Tools on macOS:

```sh
git clone https://github.com/OthmaneBlial/droidless.git
cd droidless
cargo build --release --locked
sh tools/fetch-notepad.sh
target/release/droidless run --headless --ephemeral --size 390x844 \
  --click "＋" --input "Hello, desktop" artifacts/apks/notepad-v1.0.0.apk
```

Normal builds and tests need no Android SDK or emulator. Linux has a headless
code path; Linux builds and native UI have not been verified.

</details>

## ⚙️ Follow the bytecode

<img src="docs/assets/runtime-path.svg" alt="APK → Rust DEX interpreter → Android API bridge → native AppKit controls; input returns to APK callbacks." width="100%">

| Crate | What it owns |
|---|---|
| `droidless-formats` | Bounded APK, binary XML, DEX and compiled-resource parsers |
| `droidless-runtime` | Register interpreter, managed objects, GC, lifecycle, framework APIs and Views |
| `droidless` | CLI, native macOS rendering and input bridge |

[Architecture](docs/architecture.md) · [DEX VM](docs/dex-vm.md) · [Framework](docs/framework.md)

## 🧰 Inside v0.3.0

| Capability | Implemented scope |
|---|---|
| 🧠 **Our own VM** | DEX 035–040; arithmetic, wide values, branches, arrays, fields, dispatch, managed continuations, Java throw/catch and retained DEX stack diagnostics |
| 🪟 **Native views** | Text, buttons, editors, nested layouts, bounded adapter grids, a targeted support-RecyclerView notes path and separate dialog panels |
| 🧭 **Navigation** | Same-APK Intents, typed Bundle extras, same-runtime Serializable references, retained Back stack, results, real Parcelable writers/CREATORs and lifecycle observers |
| 📓 **Persistence** | Isolated SharedPreferences and the public Notepad's SQLite save/edit/restart workflows |
| 🖼️ **Images** | Packaged XML, typed resources, PNG/JPEG/WebP decoding, granted read-only document streams and native ImageViews |
| ⏱️ **Scheduled work** | Main Handler/Looper/Message queue, guest Timer tasks, deferred workers, executor/Future results and main-thread delivery |
| 🗂️ **Java foundations** | Bounded collections, guest equality, reflection, primitive metadata, weak references and handle-based mark/sweep GC |
| ✂️ **Layout and input** | Measured text, UTF-16 ellipsis metadata, host-font ascent/descent, full accessibility labels for shortened text, clipping, focus and single-pointer gesture contracts |
| 💾 **Bounded file I/O** | App-private reads/writes and package-isolated external files; staged output commits atomically up to 64 MiB; input/output channels share position and close state, with bounded transfers |
| 🧩 **Everyday APIs** | Toast feedback, filtered file listing, selected reflected fields, UTF-8 form URLs, file-extension MIME lookup and fragment containers |

These are bounded profiles with explicit ceilings. The unchanged Notepad APK now
creates a byte-exact `.nbu` database backup and restores a tampered database; a
fresh process shows the original note and folder rows. Its post-restore
`System.exit(0)` call is still unsupported, so the restore run reports that
shutdown boundary after the data has been recovered. Custom Bundle serializables
are same-runtime references, not Java serialization or durable snapshots.
Physical public folder/dialog input, Android font/pixel parity, GIF animation
and usable slideshow remain unverified. Broad AndroidX/Compose, JNI, networking,
JIT and games are future work. [Exact compatibility](docs/compatibility.md) · [Storage limits](docs/storage.md)

<details>
<summary><strong>🔎 Inspect, replay and trace</strong></summary>

```sh
target/release/droidless inspect app.apk
target/release/droidless manifest app.apk
target/release/droidless dex app.apk
target/release/droidless classes app.apk
target/release/droidless methods app.apk
target/release/droidless resources app.apk

target/release/droidless run --headless --ephemeral fixtures/generated/intents.apk \
  --click "Open detail" --back
target/release/droidless run --headless --ephemeral fixtures/generated/scheduling.apk \
  --click "Start timer" --advance-ms 1500 --advance-ms 1500 --advance-ms 1500
```

Use `--trace-bytecode`, `--trace-methods`, `--trace-framework` and
`--trace-lifecycle` to follow the real execution path. `--stats` and
`--heap-stats` report measured counters. `--click`, `--tap`, `--key`,
`--back`, `--input-at`, `--focus-at` and `--menu-item` replay guest input.
`--advance-ms` advances the deterministic headless clock.
[CLI reference](https://othmaneblial.github.io/droidless/docs.html#commands).

</details>

## 🧪 Small lab. Executable evidence.

**150 Rust tests**, warning-free Clippy, optimized builds, five native component
checks and **4,096 seeded parser mutations** run locally. The public APK replay
checks the original app flows and certifies one unchanged CLI SHA-256. Release
packaging extracts the archive into a clean directory and checks Counter,
navigation/Back, UTF-8 preferences and original Notepad save/restart.

```sh
# CI runs on your machine. GitHub Actions stays disabled.
sh tools/ci.sh
sh tools/fetch-kascalc.sh                 # Earlier regression fixture
sh tools/fetch-simple-calculator.sh
sh tools/fetch-notepad.sh
sh tools/fetch-swpieview.sh
python3 tools/compatibility.py
python3 tools/package-macos.py
```

[Evidence catalog](compatibility/catalog.json) · [Verification record](docs/verification.md) ·
[Changelog](CHANGELOG.md)

<details>
<summary><strong>🔧 Rebuild fixtures or a development app bundle</strong></summary>

```sh
python3 tools/build-fixtures.py --sdk "$ANDROID_SDK_ROOT"
sh tools/macos-app.sh
```

SDK tools compile fixtures only. The app bundle is unsigned and intended for
local development.

</details>

## 🗺️ On the road to 50%

The **20% milestone** demonstrated an unmodified APK rendering, accepting input,
executing its own logic and updating native UI. Everyday-app workflows now add
multiple screens, persistent notes, folders, images and scheduled callbacks.

**50% remains the active target.** These labels describe project milestones,
not measured Android API coverage. Each step needs a working app flow and a
clear compatibility boundary. [Follow the roadmap](docs/roadmap.md).

## 🔐 Bring trusted APKs

DROIDLESS is an experimental runtime, not an audited security sandbox.
Package storage is confined to the host-selected app root; external storage is
a package-isolated virtual directory. Granted document access is bounded and
read-only. General file access, networking and native-library/process APIs
remain unavailable. [Security boundaries](docs/security.md).

---

<div align="center">

<img src="docs/assets/droid.svg" alt="DROIDLESS robot explorer" width="84">

**Built in Rust. A little stubborn by design. 🤖**

Apache-2.0 · Third-party artifacts retain their upstream licenses.

[Website](https://othmaneblial.github.io/droidless/) · [Notebook](https://othmaneblial.github.io/droidless/docs.html) · [Release](https://github.com/OthmaneBlial/droidless/releases/tag/v0.3.0)

</div>
