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

The earlier capture recorded the real 12.0 state; the public showcase now uses
the separate Simple Calculator APK below. Gradients/ripple rendering is partial.
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
generic Java collections and the actual subsequent failures. This records the
first storage checkpoint; the following increment moves that startup boundary.

## Current source: Java collections

The authored Collections APK executes its own conformance code and emits
`Collections passed` in the headless View snapshot. It checks null/String/custom
guest equality, Set and Map returns, iterator removal/exhaustion/fail-fast errors,
live read-only views, mutation during equals callbacks, and canonical Class keys/
package metadata. Rust integration checks retain iterator owners and Map keys/
values across GC, and enforce 16,384-entry limits without changing the store.
No native collection-fixture interaction is claimed.

Full local CI passes 24 Rust tests, 4,096 parser mutations, warning-free Clippy/
release build and all ten unchanged KasCalc scenarios.

The same unmodified Notepad APK and SHA-256 were retried with the release build.
HashSet/HashMap setup and package metadata now resolve. Application.onCreate
reaches `Lcom/raizlabs/android/dbflow/config/FlowManager;->a(Lcom/raizlabs/android/dbflow/config/g;)V`,
PC `0x0004`, and fails on
`Ljava/lang/Class;->forName(Ljava/lang/String;)Ljava/lang/Class;`.
The parent Application method is at PC `0x0016`. No Activity/UI or working notes
workflow was reached. Dynamic class loading is the next observed blocker; the
APK remains unchanged and is not redistributed.


## Current source: APK-local reflection and inherited fields

The authored Reflection APK executes lookup, canonical Class identity, arrays,
component initialization rules, no-argument construction/access checks, unwrapped
constructor errors, sticky initialization failures and inherited field aliases.
Object/int/wide/static fields and shadowing are exercised by compiled guest code.
Rust integration checks add GC identity, bounded malformed array names and
catchable missing/wrong-kind field faults. The same ReflectionContract passes on
Homebrew Java 17.0.19 after compiling with Java 8 source/target. This is desktop
Java differential evidence, not an Android reference run.

Retrying the unchanged Notepad release with the current interpreter gets through
Class.forName and Class.newInstance. GeneratedDatabaseHolder's inherited
`typeConverters` field now shares its parent's storage. At that checkpoint startup failed on
`Ljava/util/ArrayList;` in `Lcom/raizlabs/android/dbflow/config/e;-><init>()V`,
PC `0x000a`, under Application.onCreate PC `0x0016`. No Activity/UI or notes
workflow has been reached. [Reflection limits](reflection.md).

## Current source: neutral public calculator

Simple Calculator 1.0, package `uk.ac.nott.cs.itxpm`, comes from
[swiftugandan's independently built APK](https://github.com/swiftugandan/Simple-Android-Calculator/blob/3ba860b281eba34f144e4e75115f0c0a06bced31/bin/verysimplecalc.apk).
SHA-256: `7c1adc93607c8511a3abd379f74765747d2ae72fb70c4ff5c471f13e94b98921`.
The pinned fetch helper verifies the original 24,317-byte APK; it is not modified,
rebuilt, repackaged or redistributed. It declares one Activity, no permissions and
no native libraries; SDK versions are unspecified in the manifest.

Native AppKit button clicks, observed through fresh accessibility states:

| Buttons | Display |
|---|---|
| 7, +, 5, = | 12 |
| c, 8, x, 8, = | 64 |
| c, 9, /, 3, = | 3 |

[Actual window capture](assets/simple-calculator-native.png) shows the first result
in a white/charcoal native window at a 192×400 logical viewport (`--size`).
This is an actual screenshot; styling remains approximate. Gradient/image
backgrounds, Android themes and table-column stretching remain incomplete.
The APK's own handlers use boxed Double values, double arithmetic, isNaN and
Long.toString. No app-specific arithmetic or output is implemented in DROIDLESS.
Native Escape emits onPause/onStop/onDestroy and exits with status 0.

Seven headless scenarios check arithmetic, decimals, multiple digits and clear.
The ten older KasCalc scenarios also still pass. Double bit preservation,
Number dispatch, NaN/negative-zero formatting, explicit unsupported overrides
and viewport bounds have regressions. Full local CI passes 28 Rust tests,
4,096 seeded parser mutations, warning-free Clippy and a release build.
The v0.1.0 archive predates this demo; use current source. The 50% everyday-app
milestone is still ahead. GitHub Actions remain disabled; checks are local only.

## Current source: ordered Java lists

The authored Collections APK now also executes ListContract: ArrayList hierarchy,
duplicates/nulls, insertion order, indexed get/set/add/remove, first/last index
lookup, guest equality, bounds with unchanged data, iteration/exhaustion, structural
changes, nonstructural replacements during search, equality callback counts and
APK remove(int) override dispatch. The same pure-Java contract passes
on Java 17.0.19 with Java 8 source/target. The extra runtime guard against mutation
during equals is tested separately in guest DEX, not presented as Java differential
evidence. No Android reference run or native collection-fixture test is claimed.

List and Set iteration share managed storage and retain their owners/elements
across GC. Rust regressions check both list insertion overloads at the 16,384-entry
ceiling without changing values/version, and reject LinkedHashMap subclasses
whose eviction hooks would otherwise be skipped. Basic LinkedHashMap operations
reuse the existing Map implementation; access-order maps, views/iterators, bulk
operations and eviction remain unsupported. [Exact collection limits](collections.md).

The unchanged Notepad v1.0.0 APK retains SHA-256
`2c35d3dc1d41d2c761b52785c591973886fb671a2cc2e7ab047ede89599db47f`.
DBFlow now constructs ArrayList and LinkedHashMap. Startup's first unsupported
call is `Ljava/lang/Thread;-><init>(Ljava/lang/String;)V`, from
`Lcom/raizlabs/android/dbflow/f/b/a/b;-><init>(Ljava/lang/String;)V`, PC `0x0000`,
under Application.onCreate PC `0x0016`. No Activity/UI or notes workflow is reached.
The original APK is neither changed nor redistributed. Thread/queue scheduling
is the next diagnosed boundary, rather than a skipped initializer.

Full local CI passes 28 Rust tests, 4,096 seeded parser mutations, warning-free
Clippy and release build; all 17 scenarios in the two original calculator APKs
still pass. The v0.1.0 archive keeps its older scope; this increment is current
source evidence. The 50% checkpoint remains active and has not been achieved.
