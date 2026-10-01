# First interactive milestone evidence

Latest local source gate: 102 Rust tests, warning-free Clippy, optimized builds
and 4,096 seeded parser mutations. Older sections retain their milestone's counts.

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
Looper delivery/priority and parallel execution remain unsupported.
The later [executor milestone](#current-source-executor-pools-and-future-results)
supersedes the executor limitation from this checkpoint.
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
SQLite database. It creates two notes in one process, checks that both rows
persist and both titles render on return to Notes, then opens a fresh process
against the same app-data directory and verifies both titles again. The probe
checks database state and the headless View tree; its `--input` helper does not
synthesize native keyboard events.

`site/assets/notepad-preview.svg` is a vector illustration of the editor, not a
captured native window. In a separate AppKit run, the original APK accepted real
keyboard input for `Native desktop test`; leaving the editor saved that title in
the APK's SQLite database. Native text entry and save are verified, while the
window's visual fidelity and an independent Android reference run remain
unverified. The 50% everyday-app checkpoint remains ahead.

`sh tools/ci.sh` passes locally with the current Rust test suite, warning-free Clippy, the
optimized release build and 4,096 seeded parser mutations. `python3 tools/compatibility.py`
passes 17 calculator scenarios plus the Notepad Notes list/editor and two-note
SQLite save/restart probes. Both rows are retained and both titles render after
Back and in a fresh process. No GitHub Actions run was used.

## Current source: packaged binary XML pulls

Verified 2026-10-01 on macOS ARM64. The authored image APK now calls
`Resources.getXml` on a packaged XML resource and checks document traversal,
tag names, depth, namespace URI/prefix, indexed attributes, text via `nextText`,
whitespace skipping via `nextTag`, empty-element detection and parser close.
Its resource IDs and string/dimension/float values now pass Resources, Theme and
Context TypedArray reads against the same compiled XML. The replay also checks
TextView text appearance with inherited 20sp size and explicit #202a36 color,
which preserves Notepad toolbar startup after XML attributes become visible.
The APK's activity_note.xml sets app:title to a space: the replay now checks
that real toolbar child and the creation timestamp, rather than the former
synthetic Notepad label. Toolbar text comes from the APK's child TextView, avoiding
a duplicate title on its container.
The compatibility replay requires its `XML pull OK` result. The formats test also
parses namespace and Android attributes from the independently built
`SmallestAPK` manifest.

The unchanged [F-Droid SwpieView 1.3.2 APK](https://f-droid.org/en/packages/org.voidptr.swpieview/)
(`org.voidptr.swpieview`, SHA-256
`7c7a17ddf254e6f7adb53786ab3928937a4de499785475278df2fe5a034f50f3`) was
launched with `--ephemeral`. AppCompat's startup call to `Resources.getXml`,
`Xml.asAttributeSet` and the pull parser now completes. The app then completes its
vector-drawable configuration check successfully: Java String hashing now
finds its delegate, resource maps provide real typed XML attributes, and the
APK's own `VectorDrawableCompat` inflater executes in guest DEX. With platform
fragment support, it also attaches and creates its APK-local ReportFragment. The
startup now also passes AnimatorListenerAdapter, manifest ApplicationInfo, timed
OverScroller construction, virtual background setting and toolbar content
descriptions. GridView inflation, BaseAdapter construction and item-listener
registration now complete. Native startup opens a real folder chooser, and
cancellation reaches the app finish path with exit 0. The subsequent document
provider increment queries and decodes three selected-folder thumbnails and
renders them natively; full-screen viewing and vector rendering remain open. [APK source](https://github.com/err4nt/SwpieView).

```sh
sh tools/fetch-swpieview.sh
target/debug/droidless run --headless --ephemeral --stats fixtures/generated/images.apk
target/release/droidless run --ephemeral --size 390x844 --trace-framework artifacts/apks/swpieview-1.3.2.apk
```

## Current source: packaged raster images

Verified 2026-10-01 on macOS ARM64 with the authored `org.droidless.images`
fixture. The APK packages 96×64 PNG, JPEG and WebP images as resources and
assets. Its original DEX exercises `BitmapFactory.decodeResource` with bounds
and sample-size options, `decodeStream`, `decodeByteArray`, XML `src`,
`ImageView.setImageResource`, `setImageBitmap`, `setImageDrawable`, and
`BitmapDrawable`; all reported dimensions match. `python3 tools/compatibility.py`
checks the decoded format/dimensions and four ImageView nodes.

The actual native AppKit window displays the PNG resource source, the resource
Bitmap, JPEG stream Bitmap and WebP byte-array Bitmap. The captured window is
`artifacts/images-native.png`. This proves the authored raster path on this Mac,
not an independent image-heavy app. Android Bitmap pixel access/transforms,
Canvas, and vector/animated image drawables remain unsupported. Encoded image
headers are bounded to 32 million pixels.

```sh
python3 tools/build-fixtures.py --sdk "$ANDROID_SDK_ROOT"
sh tools/ci.sh
python3 tools/compatibility.py
target/release/droidless run --headless --ephemeral fixtures/generated/images.apk
```

The Intents APK now also checks platform fragments without Views: commit queues
work, explicit execution returns a meaningful boolean, tag lookup compares String
values, and Activity/argument identities survive guest GC. Its callbacks check
creation ordering and resumed/paused state. Navigation and teardown deliver the
expected callback sequence; destroyed managers release fragment references.
Late additions execute at the next host poll. Six guest failure checks cover
repeat commit, changed tags, active arguments, duplicate add, recursive execution
and a throwing callback. Existing Activity/observer checks still pass. These are
authored headless contracts, not a native fragment UI or Android reference run.

The Images fixture now also runs authored widget contracts: manifest metadata
identity/copy/default SDK checks, listener adapter interfaces and virtual callbacks,
background override/clear with guest GC, and content-description identity,
equality, empty/null semantics and accessibility importance. The Rust widget replay
checks linear/default timed scrolling, half rounding, large float deltas,
force/abort/final states, zero duration, and throwing-interpolator root cleanup.
These are deterministic headless checks; fling physics and animation delivery
remain unsupported.

A fresh native AppKit run exposed four image nodes and the exact labels
`Packaged PNG source` and `Decoded PNG image` through the accessibility tree.
The four raster images remained visible and closing the window exited with code 0.
This is authored-fixture native evidence, not a usable SwpieView workflow.

## Current source: adapter-backed photo grids

Verified 2026-10-01 with the authored `org.droidless.grids` APK. Compiled DEX
creates seven PNG cells through BaseAdapter, checks reverse observer delivery
and self-removal, and runs GC during XML inflation, binding and item callbacks.
The Rust replay checks auto-fit, all four stretch modes, 64-bit row IDs,
replacement/invalidation, stale-observer detachment, limits and recovery from
binding/click faults, including temporary-cell root release.

The native AppKit window displays seven labeled images. Clicking Photo 3 shows
`Selected photo 3 · id 3000000002`; clicking disabled Photo 2 leaves that value
unchanged. Refresh replaces the cells with four images, and Photo 4 then shows
`Selected photo 4 · id 3000000003`. Normal close exits 0. An initial host crash
was traced to casting AppKit's own gesture recognizer; the bridge now finds and
updates only its own recognizer and preserves the system recognizers.

Full local CI passes 79 Rust tests, Clippy, the optimized build and 4,096 seeded
parser mutations. The optimized compatibility replay adds grid click/refresh
checks alongside all 17 calculator scenarios, images and Notepad two-row
save/restart checks. GitHub Actions remain disabled; the 50% checkpoint remains ahead.
The subsequent activity-result increment opens SwpieView's native directory
chooser; document access and its image-viewer workflow remain unsupported.

```sh
cargo test -p droidless-runtime --test grids
sh tools/ci.sh
python3 tools/compatibility.py
target/release/droidless run --ephemeral fixtures/generated/grids.apk
```

## Activity results and host-selected folders

This records the earlier `fea6961` increment; the document workflow below
supersedes its public-app blocker.

Verified 2026-10-01 with the compiled `org.droidless.results` APK and current
optimized binary. Native Open child → Return result displays
`Result 7:-1:at finish:image/png`; Pick folder opens NSOpenPanel and selecting
`examples/results` displays `Result 404:-1:content://droidless.documents/tree/0`.
Closing the window exits 0. The optimized compatibility replay also checks child
return and Back cancellation, alongside all 17 calculator scenarios, images,
grids and Notepad two-note save/restart.

Two compiled Rust checks exercise request codes, negative requests, result
snapshots at finish, ignored finished callers, deferred stopped-caller delivery,
result-before-resume order, callback-failure teardown and GC root release.
They also check canonical launcher metadata, virtual Intent constructor setters,
action interning, data/type clearing, Bundle copies, stable attached window
tokens and null detached tokens. Folder checks cover cancellation, an invalid
file choice preserving the request, actual directory grants and the 64-grant
ceiling. No Android reference device run is claimed.

The unchanged SwpieView APK opens the native chooser; cancellation executes its
finish path and exits 0. Positive native selection for this public APK was not
completed because the UI tool lost access to the dialog. A direct host-API probe
selects `examples/images/assets` through the same completion method and reaches
the app's real `onActivityResult` in classes.dex at PC 0x000e, where it stops at
`getContentResolver()Landroid/content/ContentResolver;`. This evidence does not
establish document enumeration, stream access or a usable image-viewer workflow.

Full local CI passes 81 Rust tests, Clippy, the release build and 4,096 seeded
parser mutations. GitHub Actions remain disabled.

```sh
cargo test -p droidless-runtime --test results
sh tools/ci.sh
python3 tools/compatibility.py
target/release/droidless run --ephemeral fixtures/generated/results.apk
```

## Independent selected-folder thumbnail workflow

This records the earlier `891eebd` increment; the viewer evidence below supersedes its next blocker.

Verified 2026-10-01 with the original checksum-pinned SwpieView 1.3.2 APK. Both
debug and optimized native macOS runs open NSOpenPanel. Selecting the repository's
`examples/images/assets` folder executes the APK's real directory query,
ImageContainer Comparable sort, BaseAdapter getView and BitmapFactory stream
decoding. The AppKit window displays three PNG/JPEG/WebP thumbnails, with the
requested CENTER_CROP scaling and padding. Accessibility state contains three
images; the debug native screenshot visibly confirms the artwork. Normal close
exits 0 in both builds. No APK modifications or Android engine are involved.

The optimized compatibility replay additionally checks three decoded images and
CENTER_CROP state in the public APK, and the authored Documents fixture's
`3 images · document streams` result. Existing calculator, image, grid, result
and Notepad two-note save/restart replays all pass. The host replay's explicit
folder argument drives the same completion API as the native picker; it does not
pretend headless mode displayed a dialog.

The compiled Documents contract also checks canonical resolver identity,
percent-encoded UTF-8 path segments and read-only Lists, stable natural and
null-comparator sorting with GC/mutation failures, seven image-scale values,
ungranted/foreign/file URIs, traversal, symlinks, hard links, nested files, stream
snapshots and close faults, Cursor close, UTF-16 search indices and oversized
stream rejection. Grants do not survive a fresh runtime. Local CI passes
82 Rust tests, Clippy, both optimized executables and 4,096 parser mutations.
GitHub Actions remain disabled.

Thumbnail selection in the optimized host replay invokes the public APK's actual
onItemClick and ImageStack.toBundle. It stops at
`Bundle.putParcelableArrayList(Ljava/lang/String;Ljava/util/ArrayList;)V` in
classes.dex at PC 0x0009. This milestone proves selected-folder thumbnails;
full-screen viewing, gesture navigation and slideshow remain unproven.

```sh
cargo test -p droidless-runtime --test documents
sh tools/ci.sh
python3 tools/compatibility.py
target/release/droidless run --ephemeral --size 390x844 artifacts/apks/swpieview-1.3.2.apk
target/release/examples/document-replay artifacts/apks/swpieview-1.3.2.apk examples/images/assets
# Reproduce the next blocker:
target/release/examples/document-replay artifacts/apks/swpieview-1.3.2.apk examples/images/assets --click-first-image
```


## Independent static-image viewer and Parcelable state

Verified 2026-10-01 with the original checksum-pinned SwpieView 1.3.2 APK.
The native AppKit run selects `examples/images/assets`, displays three thumbnails,
and opens each JPEG, PNG and WebP image in the APK's full-screen Activity. Actual
screenshots visibly show the selected artwork. Escape returns to all three
thumbnails after each visit; normal window close exits 0. No screenshot artifact
is bundled for this observation. The APK is unmodified. The optimized native binary also completes folder selection,
JPEG viewer display, Escape back to three images and normal close with exit 0.

The guest onItemClick builds its ImageStack Bundle. Activity launch executes each
ImageContainer.writeToParcel and CREATOR.createFromParcel in DEX, reconstructing
its URI and MIME type. ImageView.setImageURI opens a granted document stream,
decodes the full image and closes the stream. The optimized host replay checks
one decoded viewer image, two Activity instances, Back to three thumbnails and
one Activity, GC and clean close. This replay uses the same host folder-completion
API as NSOpenPanel; it does not display a headless dialog.

The authored Parcels APK verifies positions/backpatching, Unicode and wide values,
nullable objects/list entries, nested Bundles, shallow copy constructors versus
transported object isolation, actual guest writers/CREATORs, source-list clearing
and System.gc during callbacks, activity results, throwing callbacks and root
cleanup. Host checks reject malformed lengths/magic/counts, truncation, oversized
buffers, invalid positions, recycled use and cyclic Bundles. This is compiled DEX
evidence; no Android-device differential run is claimed.

Full local CI passes 83 Rust tests, Clippy, optimized builds and 4,096 seeded parser
mutations. Public calculator, images, grids, result and Notepad save/restart
replays remain passing. GitHub Actions stay disabled. Gesture recognition, GIF
animation, slideshow, auto-hide, full styling, Binder/file descriptors, Java
serialization, custom loading and general Parcelable coverage remain ahead.

```sh
cargo test -p droidless-runtime --test parcels
sh tools/ci.sh
python3 tools/compatibility.py
target/release/examples/document-replay artifacts/apks/swpieview-1.3.2.apk examples/images/assets --click-first-image
target/release/examples/document-replay artifacts/apks/swpieview-1.3.2.apk examples/images/assets --click-first-image --back
target/release/droidless run --ephemeral artifacts/apks/swpieview-1.3.2.apk
# Native: choose examples/images/assets, click an image, press Escape.
```


## Single-pointer gestures and native tap controls

Verified 2026-10-01 with the same unmodified, checksum-pinned SwpieView 1.3.2 APK.
The optimized document replay sends real runtime DOWN/MOVE/UP events to the
Activity and its GestureDetector. The APK's onFling selects JPEG → PNG → WebP
and previous images; comparisons check exact encoded image bytes after every
transition, first/last bounds and collection. Confirmed taps execute the APK's
hide/show callbacks and restore its original controls without changing the image.

The optimized native AppKit run selects the owned assets folder, opens the first
JPEG, hides controls with an actual mouse tap, then restores them with another.
The delayed show completes with Start and Start Slideshow visible again. Escape
returns to three thumbnails; normal window close exits 0. Screenshots and native
input traces confirm the tap flow; no screenshot artifact is bundled. Native
mouse-drag verification remains pending: the UI automation tool repeatedly
returned windowNotFoundAtPosition before sending a drag. This is a tool limitation,
not native swipe evidence; only host replay proves image navigation here.

The authored Touch APK exercises local/raw coordinates, MotionEvent copies and
recycling, pointer metadata, time validation, child target capture and translation,
OnTouch precedence, default clicks, disabled/outside-drag behavior, view translation
and opacity, show/long press, confirmed/double taps, scroll/fling, paused-drag
suppression, cancellation, System.gc in callbacks and throwing callback cleanup.
Invalid host events fail, pending timers are canceled after failure and the next
stream works. The same compiled contract is rechecked after the final root-lifetime
review. No Android-device differential result is claimed.

Local CI passes 84 Rust tests, warning-free Clippy, optimized builds and 4,096
seeded parser mutations. Compatibility replay passes both public calculators,
images/XML, grids, activity results, folder streams, SwpieView viewer/Back/swipes/
taps and two-note Notepad save/restart. GitHub Actions remain disabled. The input
profile is single pointer with a bounded linear velocity estimate; multi-touch,
full interception, Android VelocityTracker parity, GIF animation, slideshow,
lifecycle auto-hide and full styling remain open.

```sh
cargo test -p droidless-runtime --test touch
sh tools/ci.sh
python3 tools/compatibility.py
target/release/examples/document-replay artifacts/apks/swpieview-1.3.2.apk examples/images/assets --click-first-image --gestures
target/release/droidless run --ephemeral artifacts/apks/swpieview-1.3.2.apk
# Native: select examples/images/assets, open a thumbnail, tap twice, Escape, close.
```

## Current source: Java timers and public slideshow diagnosis

Java Timer/TimerTask now uses one stable guest worker per Timer. The compiled
Scheduling contract checks deferred deadlines, long/Date schedules, past-Date
clamping and fixed-rate catch-up, Thread identity/name/daemon/ThreadLocal values,
serial tasks across a queue wait, cancellation during an active wait, purge,
task reuse, null/negative/overflow faults, uncaught task termination, GC, queue
capacity recovery, runtime close and TimerTask-to-main Handler UI delivery.
Its portable validation/serial-task entry point also passes on desktop Java 17
with Java 8 source/target. This is not an Android reference differential run.

The original SwpieView 1.3.2 APK retains its pinned SHA-256 and is not modified.
FrameLayout XML/parameter gravity now places its bottom controls below the toolbar;
a layout regression checks center/right/bottom/default positions, padding,
margins and parameter precedence. Actual root touch replay reaches the APK's
slideshow OnTouchListener. DOWN starts a real Timer and UP cancels it: advancing
20 seconds executes no slideshow task and preserves the original image. A
separate held-DOWN replay delivers its TimerTask on the worker, where its direct
UI call is rejected before changing the image. The failure leaves clean managed
frames; cancellation, a subsequent poll and normal close succeed. These checks
diagnose the APK's behavior and do not establish a usable slideshow.

The optimized native Scheduling fixture at 420×720 displays Background timer
queued after Start background timer, then Background timer done: 3 after its
worker tasks post results to the main Handler. Restarting and clicking Cancel
background timer leaves Background timer cancelled beyond later deadlines.
Normal window close executes teardown and exits 0.

Local CI passes 86 Rust tests, warning-free Clippy, optimized builds and 4,096
seeded parser mutations. GitHub Actions remain disabled.

Headless System.currentTimeMillis and Date now follow the deterministic clock.
Without manual advancement, Notepad's creation and formatting Dates coincide,
so its actual PrettyTime bytecode displays Created moments from now. The replay
checks that zero-duration label, both editor fields and unchanged two-title
save/restart evidence. This is the library's documented
[zero-duration format](https://www.ocpsoft.org/prettytime/); native mode still
uses the host wall clock.

Timer finalization, daemon/non-daemon JVM process-liveness parity, wall-clock
jump rebasing and full Android timing parity remain open. The executor is serial
and bounded; worker UI access remains rejected. Native drag verification and
GIF animation remain open. The v0.1.0 archive predates this source milestone.

```sh
cargo test -p droidless-runtime --test timers --locked
python3 tools/build-fixtures.py --app scheduling
java -cp examples/scheduling/build/classes org.droidless.scheduling.TimerContract
target/release/droidless run --headless --ephemeral fixtures/generated/scheduling.apk \
  --click "Start background timer" --advance-ms 1500 --advance-ms 1500 --advance-ms 1500
# JSON View text: Background timer done: 3
target/release/examples/document-replay artifacts/apks/swpieview-1.3.2.apk examples/images/assets --click-first-image --slideshow
target/release/examples/document-replay artifacts/apks/swpieview-1.3.2.apk examples/images/assets --click-first-image --slideshow-hold
sh tools/ci.sh
python3 tools/compatibility.py
```

## Current source: executor pools and Future results

Verified on 2026-10-01, macOS ARM64. The former inline Executor.execute and
fabricated shutdown stubs are removed. Single/fixed/cached factories now queue
work on reusable guest workers with stable Thread identity and ThreadLocal state.
FutureTask Callable/Runnable bodies use managed continuations and share the
existing bounded serial executor; no parallel CPU execution is claimed.

The compiled FutureContract checks deferred submission, FIFO single-pool work,
fixed-pool progress while another task blocks, cached reuse/60-second retirement,
Callable values and Runnable null/preset results, one-shot/manual FutureTask run,
Thread-dispatched FutureTask waits, virtual done callbacks and protected setters.
Checks also cover pending-main rejection, completed/zero-time main reads, worker
get/awaitTermination deadlines and interrupts, sub-millisecond rounding, null
TimeUnit, cancel before/during execution with and without interruption, original
ExecutionException cause identity, GC, shutdown rejection/draining, shutdownNow
queued-object identity without automatic cancellation, uncaught execute-worker
replacement, unsupported-task diagnostics, queue capacity, and runtime close
before/after body dispatch. Closing drops continuations without guest callbacks.
The portable FutureContract entry point passes on desktop Java 17 with Java 8
source/target; this is not an Android reference differential run.

The optimized native Scheduling APK at 420×720 verifies Start future worker →
Future waiting for input, then Deliver future input → Future result: payload.
The actual Callable blocks on its input queue and its done override posts through
the main Handler. Restarting, canceling the blocked Future, and delivering input
again leaves Future worker cancelled. Normal window close tears down the pool
and exits 0. A separate compiled unsafe-UI check rejects worker View mutation
before changing the label and leaves clean managed frames.

Local CI passes 88 Rust tests, warning-free Clippy, optimized builds and 4,096
seeded parser mutations. The unchanged public calculator, Notepad and SwpieView
compatibility replays still pass; GitHub Actions remain disabled. Pool queues
are capped at 16,384 entries and the runtime at 64 workers. Custom pool/ThreadFactory
configuration, scheduled executors, invokeAll/invokeAny, runAndReset, main waits,
synchronous native-bridge suspension and JVM finalization/liveness remain open.
The v0.1.0 archive predates this source milestone. No independently completed
asynchronous APK workflow or achieved 50% checkpoint is claimed.

```sh
python3 tools/build-fixtures.py --app scheduling
java -cp examples/scheduling/build/classes org.droidless.scheduling.FutureContract
cargo test -p droidless-runtime --test executors --locked
target/release/droidless run --headless --ephemeral fixtures/generated/scheduling.apk \
  --click "Start future worker" --click "Deliver future input"
# JSON View text: Future result: payload
target/release/droidless run --ephemeral fixtures/generated/scheduling.apk
# Native: Start future worker, Deliver future input; restart, cancel, deliver; close.
sh tools/ci.sh
python3 tools/compatibility.py
```

## Current source: editing an existing public note

Verified 2026-10-01 with the unchanged pinned Notepad 1.0.0 APK. The local
compatibility replay creates two notes and restarts, clicks the first note's
non-clickable title through its owning row's real listener, and verifies that
the editor loads that existing note. It changes the title and second editor
field, then dispatches Back through the APK. The refreshed list contains the
revised title and the untouched second note. SQLite still contains two rows;
the edited row retains its ID. A fresh process reopens that row with the exact
revised title and multiline body in its two EditTexts.

`--input-at INDEX TEXT` selects an enabled visible EditText by zero-based View
tree order. Both CLI input helpers edit the View model. The text selector chooses
the nearest clickable ancestor of a matching label; direct guest performClick
still does not bubble. Hidden branches and disabled click owners are checked.

This flow adds boxed Integer/Long/Double/Boolean Bundle extras, ordinary Parcel
primitive tags for boxed values, stable object-array sorting and bounded
TextUtils UTF-16 search/plain-text replacement. Compiled checks cover aliases,
wrong-type defaults, mixed lists, range errors, guest callback faults, mutation,
GC and ceilings. ArraySortContract's portable contract also passes on desktop
Java; this is separate from Android-specific Bundle and text behavior.

The body probe uses multiline text without XML metacharacters. An additional
probe containing `&` and `<` is not a successful round trip: inspection of the
unchanged APK shows its serializer replacing newlines but storing those
characters unescaped. At this increment its SAX loading path fails, then reaches
unsupported printStackTrace; the retained-diagnostics increment below now verifies
the APK's actual error fallback. This is not a successful text round trip or
Android-device differential evidence. SAXException retains its real Exception/Throwable
ancestry. Rich formatting, drawing and full visual fidelity remain open.

UNSPECIFIED measurement now preserves intrinsic card height, including nested
MATCH_PARENT children; compiled checks also retain EXACTLY-zero and AT_MOST
bounds. FrameLayout takes maximum child extents on both axes. VelocityTracker
uses the shared bounded linear gesture estimator, with units/clamps, duplicate
timestamps, long-clock precision, event lifetime, GC and recycled-use checks.
The base ViewGroup/ViewParent nested-scroll start callback declines the request;
accepted nested scrolling remains unsupported.

Native existing-note selection/editing is now verified separately from the CLI
input helpers. In an AppKit window, a real mouse click on the existing note's
row opened its editor. Native keyboard input replaced the title with
`Native final note` and the body with `Native final body.`. Escape dispatched
the APK's Back callback and returned to Notes with both strings visible.
SQLite contained exactly one row, ID 1, with those exact values. After a clean
exit and a fresh native process, another mouse click reopened both exact fields.
Both native processes closed with exit status zero. These are controlled manual
checks, not an automated native UI suite or Android reference-device comparison.

Inherited APK onMeasure/onLayout callbacks now run before painting and root
touch DOWN. Closed drawer children keep their zero-width guest layout and are
not painted through an invisible ancestor. AppKit hitTest receives superview
coordinates while guest MotionEvent retains flipped content coordinates, keeping
editor clicks on native focus/selection. The compiled CustomLayout contract
covers geometry, XML parameters/merge/ViewStub, state metadata, collection/fault
cleanup and the bounded framework calls needed by those layouts.
[Exact layout and styling limits](framework.md#guest-layout-callbacks-and-xml-metadata).

Local CI passes 95 Rust tests, warning-free Clippy, optimized builds and 4,096
seeded parser mutations. The optimized public compatibility replay verifies the
existing-row edit/restart alongside the unchanged calculator and image checks.
GitHub Actions remain disabled. The v0.1.0 archive predates this source milestone;
no achieved 50% checkpoint is claimed.

```sh
sh tools/ci.sh
python3 tools/compatibility.py
```

## Current-source worker Looper delivery

Prepared worker Loopers now deliver queued Messages and posted Runnables through
managed DEX continuations. The compiled WorkerLooperContract checks two worker
owners alongside main delivery, equal-deadline FIFO order, delayed deadlines,
cancellation, payload GC, callback precedence, dispatch overrides and immutable
queue ownership after Message.setTarget. A callback can wait on queue input and
resume with the same payload; empty queues and interrupted idle Loopers execute
no additional guest instructions. Native no-op dispatch remains slice bounded.
quit/quitSafely, rejected new posts, caught callback failure with loop reentry,
and close during a blocked callback are also checked.

Authored Scheduling buttons now exercise waiting, input delivery and cancellation
in headless replay. The result is `Looper result: kept-payload:input`; cancelled
completion leaves `Looper worker cancelled` and the worker is dead. A separate
native-host replay prequeues Start Looper worker and Finish later, opens its AppKit
window, dispatches the live-clock finish, runs onPause/onStop/onDestroy and exits 0
while cancelling a blocked callback. Manual native clicks on the new controls
remain unverified because the UI service could not attach to the visible window.
This does not establish Android reference parity or a public APK's worker Looper
workflow. Earlier native timer/Future and public Notepad checks remain separate.

Local CI passes 96 Rust tests, warning-free Clippy, optimized builds and 4,096
seeded parser mutations. Optimized public calculator, image and Notepad replays
remain passing. GitHub Actions are disabled; the 50% checkpoint remains ahead.

```sh
sh tools/ci.sh
cargo test -p droidless-runtime --test loopers --locked
target/release/droidless run --headless --ephemeral fixtures/generated/scheduling.apk \
  --click "Start Looper worker" --click "Deliver Looper input"
target/release/droidless run --ephemeral fixtures/generated/scheduling.apk \
  --click "Start Looper worker" --click "Finish later" --trace-lifecycle
```

## Current-source worker sleep and join

Thread.sleep(long/long,int) and Thread.join(no-arg/long/long,int) now use the
existing shared completion waits. Guest frames, local values and held monitors
survive suspension and GC. Compiled checks prove delayed wakeup, actual Thread
termination, timed/sub-ms/self/indefinite joins, interrupts and cleared flags,
inactive join on main, catchable timeout argument errors and shutdown cleanup.
The sleeping Callable check supplies input before its deadline: its Future stays
incomplete until the sleep finishes. The original unsupported-task Future guard
now uses getStackTrace, which remains unsupported, rather than sleep.

ThreadWaitContract also passes on desktop Java with a separate driver. Android
reference-device timing is not verified; sub-ms waits round up on the runtime
clock. Main positive sleep/live join and native bridge/initializer suspension
remain explicit diagnostics. The original Notepad includes sleep/join calls, but
its verified save/edit/restart flows do not establish a completed worker workflow
using these new APIs.

The authored Sleep, join and finish control now has separate automated native
host evidence. The CLI prequeues the click with time frozen, so the sleeper and
joiner remain blocked until AppKit opens and the live clock advances. Guest code
then validates the kept value, completed join, released monitor and main Thread
identity. The log records a visible native window, its SleepJoin completion,
onPause/onStop/onDestroy and native exit in that order, with process status 0.
Headless replay verifies the label still waits at 9 ms and the Activity finishes
at 10 ms. Manual native mouse input and painting the result before immediate
finish are not claimed. These remain authored checks, separate from public APK
compatibility and Android reference-device comparison.

Local CI passes 97 Rust tests, warning-free Clippy, optimized builds and 4,096
seeded parser mutations. The original public APK compatibility replays remain
passing. GitHub Actions stay disabled; the 50% checkpoint remains ahead.

```sh
sh tools/ci.sh
cargo test -p droidless-runtime --test thread_waits --test executors --locked
java -cp examples/scheduling/build/classes org.droidless.scheduling.ThreadWaitContract
python3 tools/compatibility.py
```

```sh
DROIDLESS_NATIVE_TRACE=1 target/release/droidless run --ephemeral fixtures/generated/scheduling.apk \
  --click "Sleep, join and finish" --trace-lifecycle
```

## Current-source retained Throwable diagnostics and Notepad error recovery

No-argument printStackTrace now prints the original guest DEX method/module/PC
locations, captured at construction or implicit Java fault creation. These remain
available after unwind and GC; fillInStackTrace refreshes them. Cause chains use
actual retained objects, honor guest getCause/toString/localization overrides,
stop cycles and enforce output/depth limits. Callback failures still propagate.
Compiled checks also verify constructor fillInStackTrace overrides and root
cleanup. ThrowableContract's separate desktop Java driver passes; DROIDLESS uses
DEX locations rather than source-file lines. Stream/writer overloads,
StackTraceElement arrays and suppression remain unsupported.

The pinned, unmodified public Notepad APK now completes its actual malformed XML
catch path. A fresh headless process creates a note with body
`Plain & <broken text`, saves through Back and returns to Notes. Its SAX fault is
printed with the original c() call's PC 0x0033. Another fresh process opens that
row and displays the APK's own !ERROR! body marker; both processes exit 0.
Opening the row leaves its ID, title and raw database body unchanged.

This verifies error recovery, not a successful XML-metacharacter round trip.
The original serializer still stores those characters unescaped. No native
interaction or Android reference-device comparison is claimed for this new
error path. The permanent public replay checks the fault log, guest error marker
and unchanged database row alongside the existing successful note/edit/restart
and calculator/image-app cases.

Full local CI passes 98 Rust tests, warning-free Clippy, both optimized builds
and 4,096 seeded parser mutations. GitHub Actions remain disabled and the 50%
checkpoint remains ahead.

```sh
sh tools/ci.sh
cargo test -p droidless-runtime throwables::tests --locked
java -cp examples/counter/build/classes org.droidless.counter.ThrowableContract
python3 tools/compatibility.py
```

## Current-source menu XML and item state

The authored MenuContract, compiled with javac/D8 into the Counter APK, verifies
flat packaged menu XML, resource/literal titles and drawable references, category
precedence and stable ordering, duplicate-ID removal, group visibility/enabled
flags and exclusive/nonexclusive checks. Invalid indices/category/action flags
reach guest catch handlers. Unsupported XML onClick fails before adding any
items; the 1,024-item cap preserves the existing menu. GC checks retain items,
listeners, icons and color lists and release them after the last menu root drops.
Drawable.setTint invokes actual guest setTintList overrides; a failing override
propagates without leaking native roots. Tint metadata is retained, not painted.

The pinned original Notepad APK's onCreateOptionsMenu callback also completes:
its AppCompat inflater delegates to the platform inflater, loads the packaged
delete item and executes its DrawableCompat tint path. This preparation probe
is not yet evidence for native menu input or actual note deletion. Submenus,
shortcuts, XML onClick, action Views/providers and theme references remain open.

Full local CI passes 99 Rust tests, warning-free Clippy, both optimized builds
and 4,096 seeded parser mutations. GitHub Actions remain disabled.

```sh
cargo test -p droidless-runtime --test menus --locked
sh tools/ci.sh
```

## Current-source foreground options and guest delete callback

The compiled MenuActivity contract verifies one creation per cached menu,
preparation on each opening, visible/disabled/checked snapshots, listener-first
selection and Activity fallback. A callback may act and return false. Invalidation
inside a listener preserves that callback's fallback, then releases the old menu.
Rejected preparation blocks stale host input while retaining the cache. Create
rejection/invalidation/failure and preparation failure recover without retaining
menu roots. Navigation/finish during callbacks is drained before exposing options;
background/destroyed Activity handles cannot select another screen's menu.

The headless driver exposes these callbacks through `--menu-item TEXT`. Native
menu presentation/input remain ahead. Class.toString now prints class/interface
names, primitive/void names and array descriptors. The compiled PrimitiveContract
and its independent desktop Java run check these forms; unknown native metadata
remains unsupported. Packaged `<view class="...">` also invokes the actual named
DEX constructor, retains objects across its GC and applies its XML attributes.

A separate pinned, unmodified Notepad probe saves two notes with distinct bodies,
opens one and selects its original Delete item. The actual callback executes
SQLite DELETE; the resulting table contains only the other note with its original
ID, title and body. EventBus description/posting and generic Snackbar-layout
construction now execute. Returning to Notes then fails at the original
TextView.getLayout call during Snackbar measurement. This is partial deletion
evidence, not a passing delete/return/restart workflow or native menu interaction.
The regression catalog keeps the complete deletion capability false.

Full local CI passes 100 Rust tests, warning-free Clippy, optimized CLI/replay
builds and 4,096 parser mutations. The public regression replay also passes the
existing calculator, image and Notepad create/edit/restart/error-recovery cases.
The separate desktop Java class-description driver passes on Java 17.0.19.
GitHub Actions remain disabled; the 50% checkpoint remains ahead.

```sh
cargo test -p droidless-runtime --test menus --test reflection --locked
java -cp examples/reflection/build/classes org.droidless.reflection.PrimitiveContract
sh tools/ci.sh
python3 tools/compatibility.py
```

## Current-source XML inflation completion and live-region state

The shared inflater invokes virtual View.onFinishInflate after applying XML
attributes and attaching all children. The base View callback is empty, following
the [API-21 inflater](https://android.googlesource.com/platform/frameworks/base/+/android-5.0.0_r1/core/java/android/view/LayoutInflater.java).
A compiled custom LinearLayout checks that its constructor sees no children,
its completion callback runs once, and its child lookup/text survive callback GC.
The existing merge-parent path does not issue a completion callback to that parent.

View.setAccessibilityLiveRegion/getAccessibilityLiveRegion retain the two mode
bits used by the [API-21 View](https://android.googlesource.com/platform/frameworks/base/+/android-5.0.0_r1/core/java/android/view/View.java).
The compiled contract checks default state, all mode bits, out-of-range values
and GC. Android accessibility-service event delivery is not implemented.

The pinned Notepad delete probe now completes its original SnackbarLayout child
binding and delivers its queued confirmation callback; it now reaches
TextView.getLayout during Snackbar measurement. The complete workflow remains
unverified, and the catalog keeps that capability false.

## Current-source Handler payload factory

Handler.obtainMessage(int, Object) reuses the existing Message factory and queued
delivery path. The compiled scheduling contract verifies the what field, Handler
target, null payload, exact object identity across GC and deferred dispatch with
its original fields. The queue alone retains the Handler, Message and payload
after the fixture drops its own roots. Ordering, cancellation, double enqueue
and worker delivery regressions still pass. This follows the [API-21 Handler
factory](https://android.googlesource.com/platform/frameworks/base/+/android-5.0.0_r1/core/java/android/os/Handler.java).

The pinned Notepad Delete callback now queues and executes its original Snackbar
Handler.Callback. It gets past SwipeDismissBehavior float clamping, then fails at
TextView.getLayout during Snackbar measurement.
Complete delete/return/restart and native options-menu input remain unverified.

## Current-source float extrema

Math.max(float, float) adds the missing counterpart to the existing float minimum.
The compiled MathContract checks both methods with ordinary/reversed/equal values,
both orders and signs of zero, infinities, either/both NaN operands and a subnormal.
Its signed-zero check compares reciprocal signs. The same contract passes on
desktop Java 17, following the [Java Math contract](https://docs.oracle.com/javase/8/docs/api/java/lang/Math.html#max-float-float-).

```sh
cargo test -p droidless-runtime --test boxed_double --locked
java -cp examples/counter/build/classes org.droidless.counter.MathContract
```

The pinned original Notepad Delete replay now gets past Snackbar float clamping
and completes detached-child removal; it reaches TextView.getLayout during
Snackbar measurement. The intended
row is removed and the other retains its original ID/title/body, but complete
delete/return/restart and native menu input are still unverified.

Full local CI passes 101 Rust tests, warning-free Clippy, optimized CLI/replay
builds and 4,096 seeded parser mutations. GitHub Actions remain disabled.

## Current-source detached ViewGroup removal

Temporary detach now removes the child from the visible array and clears its
parent, following the [API-21 ViewGroup contract](https://android.googlesource.com/platform/frameworks/base/+/android-5.0.0_r1/core/java/android/view/ViewGroup.java).
Reattachment restores the parent, index and layout parameters without issuing
another hierarchy-added callback. Permanent removeDetachedView runs the actual
APK onViewRemoved override; its base implementation notifies the hierarchy listener.
The compiled contract checks remaining-child identity, suppressed temporary-detach
notifications, reattachment, callback GC and retained state after a thrown override.
Native checks reject active child touch capture and LayoutTransitions without
changing the remaining array. Window attachment and disappearing-view animations
remain unsupported; this is the unanimated removal profile.

The pinned, unmodified Notepad Delete replay now passes RecyclerView detached-child
removal and reaches Snackbar measurement at TextView.getLayout. It still removes
only the intended SQLite row and preserves the other's original ID/title/body.
Complete delete/return/restart and native options-menu input remain unverified.

```sh
cargo test -p droidless-runtime --test menus --locked
sh tools/ci.sh
python3 tools/compatibility.py
```

## Current-source measured text Layout

The compiled TextLayoutContract uses an APK TextView subclass whose onMeasure
collects before and after its real superclass call. It verifies null before
measurement; calculated wrapping and explicit/trailing newlines; width and owned
text; stable identity across unchanged measurement and GC; invalidation after
text, size, padding and single-line changes; height limits; zero width/size; empty
and supplementary-character text; an exception after measurement; and editable
append updating the owning View. Old Layout text remains an owned snapshot.
One Rust check runs that DEX contract, checks clean frames and verifies temporary
roots are released. These are bounded engine checks, not Android pixel/font parity.

```sh
cargo test -p droidless-runtime --test widgets --locked
sh tools/ci.sh
python3 tools/compatibility.py
```

The original pinned Notepad replay again creates two rows and deletes only the
intended row, preserving the survivor's original ID/title/body. getLayout now
resolves; the original Snackbar raises a guest NullPointerException at
onMeasure PC 0x0036 when getLineCount receives null, because its native container
has not measured the child. Container measurement and complete delete/restart
remain open; no successful public Delete or new native result is claimed.

Full local CI passes 102 Rust tests, warning-free Clippy, optimized CLI/replay
builds and 4,096 seeded parser mutations. All existing public APK compatibility
replays pass. GitHub Actions remain disabled.

## Current-source container child measurement

The compiled text contract additionally nests an APK LinearLayout subclass in a
FrameLayout. Its callback reads the real measured child Layout after super; the
children collect during measurement. It checks weighted width distribution,
explicit layout-parameter margins, frame padding, final MATCH_PARENT width,
measured height, GONE suppression and ancestor layout requests after text changes.
A throwing child leaves the frame dirty and a retry succeeds. The Rust check
verifies exact rendered bounds and rejection of hierarchy mutation during a child
callback, clean frames and released temporary roots. EXACTLY, AT_MOST and
UNSPECIFIED weighted measurement pass. Remeasurement at unchanged bounds still
runs onLayout, an unchanged snapshot skips duplicate layout, and a thrown layout
callback can be retried without another request. An authored support-class probe
uses a differently named animator type and collects in its setter, proving the
unanimated profile is applied before onMeasure through DEX signature lookup. This is engine behavior
evidence; Android baseline/font/pixel parity is not claimed.

The pinned original Notepad Delete replay now completes Snackbar text measurement
and reads Layout.getLineCount from its actual measured TextView. It reaches the
original LinearLayout.getOrientation call and next stops at View.isPaddingRelative
through SnackbarLayout.a(III)Z at PC 0x001e. The intended SQLite row is removed and the survivor retains its original
ID/title/body. Complete delete/return/restart and native menu input remain open.

Full local CI passes 102 Rust tests, warning-free Clippy, optimized CLI/replay
builds and 4,096 seeded parser mutations. All existing public APK compatibility
replays pass. GitHub Actions remain disabled.

## Current-source relative padding state

The same compiled text contract checks default physical padding, relative
start/end setters and getters, invalidated text width after padding changes,
switching back to physical padding, XML start/end values including a single
relative edge, per-View state isolation and collection. Its Rust check rejects
negative input before changing the retained padding or relative state.
These checks cover the current LTR profile; RTL and legacy target-SDK padding
precedence remain unsupported.

The pinned, unmodified Notepad replay creates two rows and removes only the
selected row through its original Delete callback. The survivor retains its
original ID/title/body. Relative padding now resolves and Snackbar completes
measurement. Its onLayout listener reaches Snackbar.b()V at PC 0x001b, then the
support animation bridge stops at View.animate()Landroid/view/ViewPropertyAnimator;.
Complete delete/return/restart and native menu input remain unverified.

Full local CI passes 102 Rust tests, warning-free Clippy, optimized CLI/replay
builds and 4,096 seeded parser mutations. All existing public APK compatibility
replays pass. GitHub Actions remain disabled.
