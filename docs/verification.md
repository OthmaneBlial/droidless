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
At the ordered-list checkpoint DBFlow constructed ArrayList and LinkedHashMap. Its first unsupported
call is `Ljava/lang/Thread;-><init>(Ljava/lang/String;)V`, from
`Lcom/raizlabs/android/dbflow/f/b/a/b;-><init>(Ljava/lang/String;)V`, PC `0x0000`,
under Application.onCreate PC `0x0016`. No Activity/UI or notes workflow is reached.
The original APK is neither changed nor redistributed. The scheduling increment
below moves this diagnosed boundary forward without skipping initializers.

Full local CI passes 28 Rust tests, 4,096 seeded parser mutations, warning-free
Clippy and release build; all 17 scenarios in the two original calculator APKs
still pass. The v0.1.0 archive keeps its older scope; this increment is current
source evidence. The 50% checkpoint remains active and has not been achieved.

## Current source: main-thread messages and native timers

The authored Scheduling APK executes queued Handler callbacks in guest DEX.
Compiled checks cover deferred/negative-delay/equal-deadline ordering,
Handler.dispatchMessage and handleMessage overrides, Callback handling, public
Message fields, cancellation identity, main Looper/Thread identity, monotonic
clock bounds and pending/active GC retention. Errors retain the guest cause and
leave clean frames; queue capacity, deadline overflow, self-posting and shutdown
have regressions. ThreadContract's unstarted metadata/manual run checks also pass
on Java 17.0.19 with Java 8 source/target. No background-thread or Android
reference differential evidence is claimed. [Exact scope](threading.md).

Actual native clicks at a 360×340 logical viewport displayed Waiting for timer,
then Timer done: 3 through three 1,500-ms callbacks. Start followed by Cancel
before the first deadline remained Timer cancelled after that deadline. Finish
later closed through a queued guest callback, emitting onPause/onStop/onDestroy
and exiting with status 0. The [native capture](assets/scheduling-native.png)
is an unedited screenshot of this authored fixture. Headless `--advance-ms`
replay independently produces Timer done: 3. The public Simple Calculator was
also rechecked after the event-loop change with native 7 + 5 = 12 and clean close.

The unchanged Notepad release still has SHA-256
`2c35d3dc1d41d2c761b52785c591973886fb671a2cc2e7ab047ede89599db47f`.
At the scheduling checkpoint Thread(String) resolved. Its first failure was unsupported class
`Ljava/util/concurrent/LinkedBlockingQueue;`, in
`Lcom/raizlabs/android/dbflow/f/b/a/b;-><init>(Ljava/lang/String;)V` at PC `0x0006`,
under Application.onCreate PC `0x0016`. No Activity/UI or notes workflow is reached.
Real transaction workers and blocking queues remain required.

Full local CI passes 32 Rust tests, 4,096 seeded parser mutations, warning-free
Clippy and release build. All 17 original calculator scenarios pass. GitHub
Actions stay disabled. This is current-source support; the v0.1.0 archive retains
its older scope. The 50% everyday-app checkpoint has not been achieved.

## Current source: immediate FIFO queue operations

The authored Collections APK additionally executes QueueContract: default/fixed
capacity, invalid capacity faults, FIFO order and duplicates, null insertion
rejection, empty/full behavior, head versus removal methods, first equal removal,
guest equals even on identical references and original equality exceptions.
Inherited add/element/remove/isEmpty dispatch APK offer/peek/poll/size overrides.
The same immediate contract passes on Java 17.0.19 with Java 8 source/target.
Its equals callbacks invoke System.gc; retained queue elements also survive an
explicit runtime collection before later removal. [Exact queue scope](collections.md#immediate-fifo-queues).

A separate regression checks the 16,384-entry resource ceiling without changing
values/version, rejection of take/put/iterator/toString without consuming data,
and explicit rejection of structural mutation during guest equals with clean
frames. These guards are implementation limits, not Java differential claims.
No blocking/concurrent queue execution or Android reference run is claimed.

The original Notepad release was retried with the new release build, retaining
SHA-256 `2c35d3dc1d41d2c761b52785c591973886fb671a2cc2e7ab047ede89599db47f`.
LinkedBlockingQueue construction now resolves. Startup reaches the DBFlow worker
start path, then stops at `unsupported Thread.start: background guest execution
is not implemented`, from `f/b/a/b.a()` at PC `0x0007`, called by `d/c.<init>` at
PC `0x000e`, under Application.onCreate PC `0x0016`. No Activity/UI or notes
workflow is reached; there is no synchronous worker substitute or skipped initializer.

Full local CI passes 33 Rust tests, 4,096 seeded parser mutations, warning-free
Clippy and release build. The 17 original calculator cases still pass. The new
queue subset is current-source support; v0.1.0 retains its older scope. The 50%
everyday-app checkpoint remains ahead and the goal stays active.

## Current source: managed DEX call continuations

DEX-to-DEX calls now use an iterative managed frame stack. A caller remains at
its invoke PC while its callee runs; return words become the caller's rooted
invoke result before move-result. Exception routing preserves the faulting and
calling PCs and unwinds only the current invocation's frame range. Native bridges
and class initialization still execute synchronously within a step.

The authored Counter APK additionally compiles FrameContract: nested reference
and wide returns, recursive throws, typed catches and finally. The normal contract
also passes on Java 17.0.19 with Java 8 source/target. A separate Rust regression
executes that DEX one instruction at a time and collects after every pause. It
checks caller/callee/result/exception roots, zero-step behavior, original guest
causes, every nested diagnostic frame, 128-frame rejection and clean stacks after
failure. This is continuation evidence, not an Android reference or worker test.

```sh
cargo test -p droidless-runtime sliced_calls --locked
mkdir -p artifacts/frame-java-contract
javac -source 8 -target 8 -Xlint:-options -d artifacts/frame-java-contract examples/counter/FrameContract.java
java -cp artifacts/frame-java-contract org.droidless.counter.FrameContract
```

Full local CI passes 34 Rust tests, 4,096 seeded parser mutations, warning-free
Clippy and release build. All 17 original calculator scenarios pass. Fresh
headless timer replay still reaches Timer done: 3. The unchanged original Notepad
digest was checked again; startup still stops at Thread.start from DBFlow's
`f/b/a/b.a()` at PC `0x0007`, before Activity/UI creation.

A fresh unsigned development bundle launched the public calculator and emitted
onCreate/onStart/onResume. Native window automation could not attach, reporting
`cgWindowNotFound` after refreshed app selection and a connection reset. The
process was confirmed running, then stopped as a test process. **The native
interaction/clean-close recheck was not completed for this increment.** Existing
calculator/timer captures document the earlier native milestones; no new capture
or native interaction claim is added here.

Thread.start, waiting workers and blocking queue waits remain unsupported; the
managed continuations are the first execution prerequisite, not a worker substitute.
The v0.1.0 archive keeps its earlier scope and the 50% checkpoint remains active.

## Current source: bounded guest workers and queue waits

Thread.start now defers APK Thread.run/Runnable execution to a named Rust host
worker executor, with a shared heap and independent managed frame stacks. Each
poll joins the host executor before main Handler/UI work. Logical workers retain
stable guest identities across executor migrations; execution remains serial,
not parallel CPU execution. A host poll shares 64 slices of 1,024 top-level steps
across worker/message dispatches, with at most 64 live workers. Synchronous native
bridges/initializers still share the overall instruction budget within each step.

The authored WorkerContract passes on desktop Java 17 and compiled DEX for worker
identity, nested take, FIFO/capacity put, interrupt flag clearing, start-once faults,
reentrant locks and monitor contention. DEX regressions collect across waits and
verify retained local strings, main Handler result delivery, rejected direct UI
mutation, worker faults/lock cleanup, native-bridge suspension diagnostics, bounded
spinning, capacity preservation and shutdown. The headless Start worker action
reaches `Worker result: kept-consumer:payload` through worker DEX and a main callback.

```sh
cargo test -p droidless-runtime --test workers --locked
mkdir -p artifacts/worker-java-contract
javac -source 8 -target 8 -Xlint:-options -d artifacts/worker-java-contract examples/scheduling/WorkerContract.java
java -cp artifacts/worker-java-contract org.droidless.scheduling.WorkerContract
target/release/droidless run --headless --ephemeral fixtures/generated/scheduling.apk --click "Start worker"
```

Full local CI passes 37 Rust tests, 4,096 seeded parser mutations, warning-free
Clippy and release build. All 17 original calculator cases pass; the timer replay
still reaches Timer done: 3. The original Notepad release SHA-256 matches its
catalog pin. At the worker checkpoint startup passed deferred Thread.start and failed at
`Integer.TYPE:Class` in generated DBFlow `d/n.<init>` PC `0x0007`, under config
`a.<init>` PC `0x005c` and Application.onCreate PC `0x0016`. No Activity/UI or
independently working notes/worker workflow has been reached.

The fresh unsigned bundle launched Scheduling and emitted onCreate/onStart/
onResume. Native automation again reported `cgWindowNotFound`; the running test
process was confirmed and then stopped with Ctrl-C. No fresh native worker
interaction, screenshot or clean-close result is claimed. Earlier native timer
captures remain historical evidence.

Main waits, native-bridge/initializer suspension, sleep/join, wait/notify, worker
Looper delivery/priority, executors and parallel execution remain unsupported.
The v0.1.0 archive retains its older scope; the 50% checkpoint remains active.

## Current source: primitive metadata, map copying and read-only lists

Verified on 2026-10-01, macOS ARM64. PrimitiveContract executes all nine native
wrapper TYPE fields in compiled DEX, checking canonical primitive identity,
Java names, wrapper lookup, constructor faults, primitive arrays, Class-key maps
and GC. Rust regressions reject final writes, wrong-kind and wrong-type field
references without changing cached metadata or leaving frames behind. The same
normal contract passes on desktop Java 17 with Java 8 source/target.

MapCopyContract checks HashMap/LinkedHashMap copies in both directions, guest
equality callbacks with GC, replacement, null keys/values, empty and self copies.
It also passes on desktop Java. A separate guest-mutation regression clears the
source inside equals and collects it: the native snapshot retains copied values,
then releases temporary roots on the terminal diagnostic. A capacity regression
confirms that bulk copying retains earlier replacements when a later insertion
hits the 16,384-entry ceiling. Custom Map copying remains unsupported.

ListContract now checks live/nested unmodifiableList views, RandomAccess metadata,
virtual get with GC, null/search behavior, mutation rejection and fail-fast native
iteration. Read-only iterators delegate rather than changing their backing
iterator's permissions; a separately held mutable alias stays writable. Wrapper
and iterator owners survive collection. This normal contract also passes on
desktop Java. No Android reference differential run is claimed.

```sh
cargo test -p droidless-runtime --test collections --test reflection --locked
mkdir -p artifacts/java-metadata-map
javac -source 8 -target 8 -Xlint:-options -d artifacts/java-metadata-map examples/collections/ListContract.java examples/collections/MapCopyContract.java examples/reflection/PrimitiveContract.java
java -cp artifacts/java-metadata-map org.droidless.collections.ListContract
java -cp artifacts/java-metadata-map org.droidless.collections.MapCopyContract
java -cp artifacts/java-metadata-map org.droidless.reflection.PrimitiveContract
```

The original Notepad APK retains SHA-256
`2c35d3dc1d41d2c761b52785c591973886fb671a2cc2e7ab047ede89599db47f`.
DBFlow's primitive metadata and native map copying now resolve, followed by
unmodifiableList construction. The fresh release build stops at unsupported
`Ljava/util/concurrent/CopyOnWriteArrayList;`, in bundled
`Lcom/b/a/c/d/a/a;-><init>()V` PC `0x0012`, under its `<clinit>` PC `0x0002`
and Application.onCreate PC `0x001e`. No Activity/UI or working-notes workflow is
reached. No initializer is skipped and the original APK is unchanged.

Full local CI passes 39 Rust tests, 4,096 seeded parser mutations, warning-free
Clippy and release build. All 17 original calculator scenarios pass. Headless
worker delivery and timer replay still produce `Worker result: kept-consumer:payload`
and `Timer done: 3`.

Browser/native automation became available again for this increment. The fresh
unsigned development bundle was checked through native Simple Calculator clicks:
7 + 5 = 12, 8 × 8 = 64 and 9 / 3 = 3. Native close emitted onPause/onStop/onDestroy
and exited with status 0. The existing white/charcoal showcase capture is retained.
A native Scheduling click on Start worker also delivered the full accessibility
text `Worker result: kept-consumer:payload`; the long visible label clips at the
360×340 test viewport. Native close emitted the same teardown and status 0.
This supplies authored native worker-to-main evidence; it does not establish an
independently working asynchronous notes app. Earlier automation failures above
remain the record of those checkpoints.

The portable site was verified over local HTTP: all 29 files match source bytes,
relative links/fragments resolve and downloaded documentation matches the repo.
Desktop and 390×844 mobile browser review confirm the neutral showcase and no
horizontal page overflow. Copy build commands was checked through a real native
Chrome paste of the expected command block.

GitHub Actions remain disabled; CI runs locally. v0.1.0 keeps its earlier archive
scope. The 50% everyday-app checkpoint remains active and has not been achieved.

## Current source: snapshot lists

Verified on 2026-10-01, macOS ARM64. CopyOnWriteArrayList uses the existing bounded
managed List store with snapshot iteration. SnapshotContract executes in the
authored Collections APK and passes on desktop Java 17 with Java 8 source/target.
It checks duplicates/nulls, indexed reads/writes, search/equality, remove during
iteration, iterator exhaustion and rejected iterator mutation. Old snapshot values
survive later set/add/remove/clear calls. Read-only List wrapping retains this
behavior. Reentrant read search keeps the original snapshot even if guest equals
clears the live List and runs GC.

The Rust integration check collects an otherwise unreachable live List while
its iterator retains the old dynamic value, then verifies that releasing the
iterator releases that value. Another check starts a guest writer, polls its
managed execution and confirms that the old iterator retains its original value
while live reads see the worker's replacement. Entry ceilings and unknown
constructors/methods fail without changing the store. A separate negative check
clears the List inside remove equality and collects it: the runtime rejects write
revalidation explicitly, preserves callback effects, unwinds frames and releases
temporary snapshot roots. This guard is not desktop Java differential evidence.

```sh
cargo test -p droidless-runtime --test collections --locked
mkdir -p artifacts/snapshot-java-contract
javac -source 8 -target 8 -Xlint:-options -d artifacts/snapshot-java-contract examples/collections/SnapshotContract.java
java -cp artifacts/snapshot-java-contract org.droidless.collections.SnapshotContract
target/release/droidless run --headless --ephemeral fixtures/generated/collections.apk
```

The desktop reference main uses Thread.join to wait for its worker. Guest join
remains unsupported; DROIDLESS uses its worker polling path. Iterator creation
copies O(n) references rather than sharing Java's backing array. No parallel CPU,
complete concurrency library or Android reference execution is claimed.
[Methods and limits](collections.md#snapshot-lists).

The unchanged Notepad v1.0.0 release retains SHA-256
`2c35d3dc1d41d2c761b52785c591973886fb671a2cc2e7ab047ede89599db47f`.
Snapshot-list construction now passes. The fresh release build stops at:

```text
Lir/cafebazaar/notepad/App;->onCreate()V [classes.dex, PC 0x002e]:
unsupported framework field Landroid/os/Build$VERSION;->SDK_INT:I
```

No initializer is skipped. Activity/UI creation and independent notes/worker
workflows remain unproven. The 50% checkpoint remains active and ahead.

Full local CI passes 40 Rust tests, warning-free Clippy, release build and 4,096
seeded parser mutations. All 17 original calculator scenarios pass. Fresh headless
checks produce Collections passed, Worker result: kept-consumer:payload and
Timer done: 3. This increment changes runtime collection behavior and documentation;
the previously verified neutral native calculator capture remains the showcase.
GitHub Actions remain disabled, and the v0.1.0 archive retains its earlier scope.

## Current source: API profile and Application observers

Verified on 2026-10-01, macOS ARM64. Build.VERSION.SDK_INT reads the fixed API-21
branch profile across GC and inherited aliases. The compiled Reflection fixture
checks canonical VERSION class lookup and avoids subclass initialization for an
inherited static read. Rust mutation checks reject writes, instance access and
wrong types without changing the native value or leaving frames behind. The
profile is independent of APK min/target SDK and host OS; it does not imply full
API-21 support. A regression also rejects APK redefinition of the native VERSION
class. This framework check is not a desktop Java differential run.

The authored Intents APK now registers Application lifecycle observers. The
compiled test checks delivery at Activity super calls, Application identity,
six event types, order and navigation/Back/close. An observer unregisters itself
and a later observer, registers a replacement and collects: the current snapshot
still delivers the removed observer, and later events deliver the replacement.
Registered observers survive GC; unregistered ones become collectible. A throwing
observer removes itself and collects before raising an exception; the fault
propagates, frames unwind and temporary snapshot roots are released.

Collecting inside callbacks initially reproduced two transition failures: a
popped pending Intent was unrooted during onPause, and an outgoing Activity could
be unrooted between lifecycle calls. Active navigation actions now remain rooted
until processing ends, including error cleanup; all registered Screen Activities
remain rooted until their Screen is removed. Existing navigation regressions and
the new observer check pass with collection at each observer event.

```sh
cargo test -p droidless-runtime --test reflection --test intents --locked
sh tools/macos-app.sh
artifacts/DROIDLESS.app/Contents/MacOS/DROIDLESS run --ephemeral --size 420x500 --trace-lifecycle --trace-methods fixtures/generated/intents.apk
```

Native accessibility editing set Home input to Observer-safe navigation. Clicking
Open detail displayed Original extras; native Escape returned to Home resume 2
with that input preserved. Native close completed with status 0. The method trace
records 33 observer interface calls: 5 created, 6 started, 6 resumed, 6 paused,
6 stopped and 4 destroyed. No Android reference run, saved-state recreation or
independent multi-screen Notes workflow is claimed.

The unchanged Notepad release retains SHA-256
`2c35d3dc1d41d2c761b52785c591973886fb671a2cc2e7ab047ede89599db47f`.
It passes SDK checks and observer registration, then stops at FileInputStream:

```text
App.onCreate PC 0x0056 -> com/b/a/a/e.a() PC 0x000a -> e.b() PC 0x0006
unsupported class Ljava/io/FileInputStream;
```

Disassembly shows that Stetho's process-name helper opens /proc/self/cmdline.
No initializer or bundled code is skipped. This remains before Activity/UI
creation and successful independent worker execution.

Full local CI passes 41 Rust tests, warning-free Clippy, release build and 4,096
seeded parser mutations. All 17 original calculator scenarios pass. The neutral
showcase is retained. GitHub Actions remain disabled; v0.1.0 retains its earlier
archive scope, and the 50% everyday-app checkpoint remains ahead.

## Current source: public Notepad editor

Verified 2026-10-01 on macOS ARM64 using the unchanged Notepad 1.0.0 release,
package `ir.cafebazaar.notepad`:
[upstream release](https://github.com/MohMah/android-notepad/releases/tag/v1.0.0).
SHA-256: `2c35d3dc1d41d2c761b52785c591973886fb671a2cc2e7ab047ede89599db47f`.
`sh tools/fetch-notepad.sh` downloads that public artifact and refuses a digest
mismatch; the APK is neither modified nor redistributed.

The original APK now completes Application startup and renders its Notes list,
including the empty state and **＋** button. Clicking **＋** runs the APK's own
listener/navigation code and builds `NoteActivity` with two editable Views and
the `Notepad` / `Created moments ago` labels. `--input "Hello, desktop"` updates the
first editable View in the emitted tree. This CLI helper writes the View model
directly; it does not synthesize Android keyboard events or prove the app's save
listener. The note editor can be replayed with:

```sh
target/release/droidless run --headless --ephemeral --size 390x844 \
  --click "＋" --input "Hello, desktop" artifacts/apks/notepad-v1.0.0.apk
```

The tree comes from the original APK and runs its Activity/DBFlow initialization.
`tools/compatibility.py` also opens the editor, changes the title, dispatches
Back through the APK, and confirms the resulting `Note` row in its private
SQLite database. Returning to Notes renders the saved title immediately. A
second DROIDLESS process opens the same app-data directory and renders that
title again. The probe checks database state and the headless View tree, not
native keyboard events.

`site/assets/notepad-preview.svg` is a vector illustration of the editor, not a
captured native window. AppKit text interaction, visual fidelity and an
independent Android reference run remain unverified. The 50% everyday-app
checkpoint remains ahead.

`sh tools/ci.sh` passes locally with the current Rust test suite, warning-free Clippy, the
optimized release build and 4,096 seeded parser mutations. `python3 tools/compatibility.py`
passes 17 calculator scenarios plus the Notepad Notes list/editor and SQLite
save/restart probes. The saved row is retained and its title is rendered after
restart. No GitHub Actions run was used.
