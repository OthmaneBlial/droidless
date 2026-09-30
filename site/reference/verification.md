# First interactive milestone evidence

Host: Apple Silicon macOS 26.6. Date: 2026-09-30. Runtime: DROIDLESS Rust interpreter
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

The current native build was also checked with Counter. Setting the NSTextField
value to Bonjour and clicking Copy input displayed Bonjour via the APK callback.
A native letter key produced `key 45` through its OnKeyListener (the host's French
keyboard layout supplied q); the handled key did not enter the text field.
Closing its native window emitted onPause/onStop/onDestroy and process exit 0.
KasCalc was rechecked through native 7 + 5 = 12.0, followed by the same lifecycle
and process exit 0. Earlier reattachment/tool timeouts were resolved with the
current executable; clipboard paste automation still timed out, so this text
evidence covers accessibility value editing and ordinary key delivery, not paste/IME.

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

The macOS ARM64 release archive is extracted into a clean temporary directory;
its bundled CLI launches the Counter APK and its DEX Increment callback returns
the expected 1. The archive excludes the third-party calculator. The package
check is part of `python3 tools/package-macos.py`.

## Current source: explicit navigation

The authored Intents fixture verifies Bundle primitive/String values and defaults,
copy isolation, exact cross-Activity lifecycle order, retained parent Views across
GC, virtual Back overrides, inactive finish, isFinishing during destruction and
clean stack teardown. Undeclared and implicit targets fail explicitly.

Native AppKit verification edited Home's input to `Native preserved input`, opened
Detail with `Original extras`, then pressed Escape. Home returned with the same
text and `Home resume 2`; the window title followed the Activity. Guarded intercepted
Escape in its APK override; its Finish button returned Home with `Home resume 3`.
Escape on Home ended the process with status 0. The headless CLI replay likewise
returned JSON null after the last Activity finished. Local CI passes 17 Rust tests,
4,096 parser mutations and all ten unchanged KasCalc scenarios. No new release
archive or third-party multi-screen compatibility is claimed by this increment.

## Current source: persistent preferences

The authored Preferences APK runs typed/staged-editor conformance in its own DEX
and exposes separate main/edit Activities. Native AppKit automation pasted
`Native notes survive restart 📓`, clicked Save note, and observed the same text
on the parent screen. Closing and restarting the native process with the same
host data root displayed the saved UTF-8 note again. Clear note displayed
`No saved note`; a fresh process confirmed that the value remained cleared.
Native close/Back emitted the expected teardown and process exit 0.

This increment fixes the earlier clipboard limitation by adding standard AppKit
Edit actions. Paste now reaches EditText and the APK reads/saves that value. Full
Android IME behavior, multiline editing and clipboard APIs remain unverified or
unsupported.

Five storage regressions cover typed values/NaN/wide values across restarts,
staging/clear order/null removal/ClassCastException, multiple packages and memory-only
runs, failed writes/retry, 1 MiB ceilings, damaged files, path traversal, symlink/
Unix hard-link rejection, replaced directories and case-colliding names/identity
markers. Full local CI passes 22 Rust tests, 4,096 parser mutations, warning-free
Clippy/release build and all ten unchanged KasCalc cases.

The unmodified Notepad 1.0.0 release, package `ir.cafebazaar.notepad`, was also
inspected and attempted headlessly. Original artifact:
[upstream release](https://github.com/MohMah/android-notepad/releases/tag/v1.0.0).
SHA-256: `2c35d3dc1d41d2c761b52785c591973886fb671a2cc2e7ab047ede89599db47f`.
It has seven Activities and no native libraries. Application.getApplicationContext
now resolves; DBFlow initialization then fails in
`Lcom/raizlabs/android/dbflow/config/h;-><init>(Landroid/content/Context;)V`,
PC `0x0003`, on unsupported `Ljava/util/HashSet;`. No Activity/UI or notes workflow
was reached. The APK was neither changed nor redistributed. Next work follows
generic Java collections and the actual subsequent failures.
