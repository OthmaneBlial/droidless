# First interactive milestone evidence

Latest local source gate: 145 Rust tests, warning-free Clippy, optimized builds
and 4,096 seeded parser mutations. Older sections retain their milestone's counts.
The [latest source increment](#current-source-public-folder-deletion-and-text-ellipses)
also passes five native component checks and the full public APK replay.

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

## Current-source timed View property animations

The authored, compiled PropertyAnimationContract uses actual ViewPropertyAnimator
instances and the shared monotonic runtime clock. At 50 ms of a 100 ms linear
batch, translationX is 50 and alpha is 0.5; the rendered snapshot carries those
exact values. At completion, translationX is 100 and alpha is zero. Automatic
start, delayed start, cancellation before/after the deadline and pending-only
cancellation execute their guest callbacks. Partial replacement keeps the other
property running and preserves counted transient state until all batches end.

The check also cancels recursively from start/cancel callbacks, restarts from an
end callback and changes the underlying progress token's interpolator. The latter
changes actual intermediate values while leaving the cached View animator's
future configuration intact. Guest callbacks/interpolators collect the heap.
Start, update, cancel, end and curve faults propagate with clean VM frames and
released transient state; runtime close releases active batches without invented
completion callbacks. Releasing guest static references and collecting reclaims
the View tree and Activity, checking temporary-root cleanup.

This is the alpha and translationX/Y profile. Standalone ValueAnimator factories,
rotation/scale/Z, repeats/keyframes, hardware layers/actions and full Android
Choreographer frame-phase parity remain outside it. Host polls advance the actual
render state; this compiled check does not claim a fresh native interaction or an
Android differential run.

The pinned, unmodified Notepad APK now gets past View.animate and starts its
original Snackbar translation/alpha callbacks. The headless replay creates two
notes with distinct bodies, invokes the APK's original Delete menu, returns to
Notes, and checks that only the chosen SQLite row disappeared. The survivor
retains its original ID/title/body, appears after a fresh process and reopens with
its exact fields. The permanent compatibility replay covers the same flow after
editing another existing row. Native menu input and the complete public timed
feedback/dismissal remain unverified; the authored clock contract above does not
substitute for those observations.

Full local CI passes 103 Rust tests, warning-free Clippy, optimized CLI/replay
builds and 4,096 seeded parser mutations. All public APK compatibility replays,
including the new Delete regression, pass. GitHub Actions remain disabled; the
50% everyday-app checkpoint remains ahead.

## Current-source native Options menu bridge

The native host now requests the foreground Activity's prepared options through
the existing runtime API and sends native item actions through its checked guest
selection path. The shared compiled menu contracts continue to cover guest
create/prepare/listener/fallback behavior, stale input, invalidation, navigation,
GC and callback faults.

The separate AppKit component contract creates a real native host, invokes its
registered menu delegate, then sends a native NSMenu item action. It checks
copied UTF-8 after the borrowed title buffer changes, enabled/checked flags,
handle delivery, repeated preparation, entry clearing and delegate/window cleanup.
It also checks that preparation is not installed before initial drawing has a
valid Rust host pointer. This is programmatic component evidence, not physical
mouse input or an original APK workflow. Run it in a macOS GUI session:

```sh
xcrun clang -fobjc-arc -Wall -Wextra -Werror -framework AppKit \
  -framework QuartzCore tools/native-menu-check.m -o /tmp/droidless-menu-check
/tmp/droidless-menu-check
```

The current unsigned development bundle opened the pinned Notepad editor against
two disposable test rows. Its trace reported a visible native window at both
390×844 and an entirely on-screen 390×720 viewport. UI control by app path, bundle
ID and display name failed with cgWindowNotFound. No mouse menu selection,
public native Delete or fresh normal-close result is claimed. The diagnostic
processes were stopped explicitly; both original test rows remained unchanged.
Physical menu input and complete public timed feedback/dismissal remain pending.

Full local CI passes 103 Rust tests, warning-free Clippy, optimized CLI/replay
builds and 4,096 seeded parser mutations. The native component check compiles with
warnings treated as errors and passes. All public APK compatibility replays,
including original Notepad Delete/return/restart/reopen, pass. GitHub Actions
remain disabled and the 50% checkpoint remains active.

## Current-source replay frames and timed Notepad feedback

Date: 2026-10-01. CLI time steps now lay out and poll the current frame before
advancing its deterministic clock. This lets guest onLayout callbacks and
automatic property animations start at the existing time. The compiled Scheduling
fixture switches to a TextView that starts its animation from onLayout; the CLI
check verifies rendered x=50 and alpha=0.5 at 50 ms. The previous development
binary, replaying the same fixture, produced x=0 and alpha=1 instead.
Runtime.advance_time continues to leave layout to its caller.

The unchanged, digest-pinned Notepad APK then exposed two real factory gaps:
Message.obtain(Handler, int, Object) at the show animation's end, and
Handler.obtainMessage(int, int, int, Object) in its timeout/dismissal path. Both
reuse the existing Message allocation and owned queues. The compiled Scheduling
check reproduces each unsupported call before its fix and verifies null defaults,
signed codes/arguments, payload identity, deferred callback delivery, duplicate
enqueue rejection and retention through GC after the fixes.

The original APK's deletion message and UNDO label appear with alpha=1 inside
the 390×844 viewport at 250 ms. Advancing another 3000 ms and then 250 ms runs its
timeout and dismissal callbacks and removes the Snackbar from the View tree.
The exact surviving SQLite ID/title/body remain unchanged. A copied original-APK
test database also passes this timed sequence in the debug CLI, exits zero and
leaves both source seed rows unchanged. The permanent public replay retains the
existing immediate Delete, fresh-process list and exact survivor reopen checks
alongside the new shown/dismissed cases.

Selecting Undo, physical native menu input, native timed feedback/dismissal and
Android differential frame timing remain unverified. The bounded property profile
does not implement full Choreographer frame-phase parity. The 50% checkpoint
remains active.

Full local CI passes 104 Rust tests, warning-free Clippy, optimized CLI/document
replay builds and 4,096 seeded parser mutations. The optimized public APK
compatibility replay passes, including the new feedback visibility, timed removal
and exact survivor checks. Site clipboard success/denial checks also pass. GitHub
Actions remain disabled; these checks ran locally.

## Current-source original Notepad Undo

Date: 2026-10-01. The unchanged pinned APK now passes Delete → 250 ms → UNDO →
250 ms → 3000 ms. Its original action listener restores the title and multiline
body, removes the Snackbar and leaves the other note's exact ID/title/body intact.
Advancing past the former timeout produces no later deletion or repeated feedback.
Fresh processes render both notes and reopen the restored title/body through the
original row callback; both rows retain their verified values after reopen.

The restored row receives ID 3 after deleting ID 1 from a two-row seed. This follows
the original [Undo listener](https://github.com/MohMah/android-notepad/blob/v1.0.0/app/src/main/java/ir/cafebazaar/notepad/activities/home/Adapter.java),
which calls note.save(), and its [auto-increment model](https://github.com/MohMah/android-notepad/blob/v1.0.0/app/src/main/java/ir/cafebazaar/notepad/models/Note.java).
The packaged DEX contains a Note INSERT that lists title/body/drawing/createdAt
columns and omits id. The observed fresh ID is consistent with
[SQLite AUTOINCREMENT](https://www.sqlite.org/autoinc.html); the survivor keeps ID 2.
The regression verifies the APK's behavior and does not rewrite its persistence.
No runtime source change was needed for this workflow.

A freshly rebuilt unsigned bundle opened the original editor against a copied
two-row database. Native tracing reported a visible 390×720 window entirely on
screen. UI control still returned cgWindowNotFound, so no physical Options,
Delete or Undo action is claimed. Only the owned diagnostic process was stopped
explicitly; both copied test rows remained exact. No fresh normal native close
result is claimed. Native Delete/Undo and timed feedback remain unverified.

The full local gate passes 104 Rust tests, Clippy, optimized CLI/document replay
builds and 4,096 seeded parser mutations. The expanded optimized public replay
passes restoration, fresh-ID, survivor, old-timeout, restart and reopen checks.
GitHub Actions remain disabled and the 50% checkpoint remains active.

## Current-source original Notepad drawer frames

Earlier checkpoint: 2026-10-01. The focus blocker recorded below was resolved by
the [next checkpoint](#current-source-focus-and-original-notepad-drawer-navigation).
Shared headless/AppKit rendering invokes virtual
View.computeScroll after layout. The Android base callback is empty; APK
overrides execute in DEX. The traversal skips hidden subtrees, follows children
after parent callbacks, skips children removed during callbacks and retains
temporary roots through GC. Callback faults propagate and release those roots.
Attached View.postInvalidateOnAnimation requests coalesce into one notification
for the next host poll; detached Views and a closed queue request none. Rendering
still uses full snapshots rather than dirty rectangles or precise vsync.

The compiled Images scroll-frame contract checks 0/50/100 positions on the shared
clock, callback counts, hidden and removed siblings, callback GC, final redraw
delivery, faults and recovery. CLI time advancement now also renders the advanced
frame before delivering the next input action. A Scheduling fixture's Back
callback observes its updated animation state; the prior CLI reported stale state.

View.hasWindowFocus now reads the host focus flag only for Views attached to the
foreground Activity's window token. AppKit samples its actual key-window state
before drawing and dispatch; headless starts false. Compiled contracts verify
host-state changes, detached Views, child Activity launch and return. Guest focus
ownership, focus-change callbacks and native editor focus synchronization remain
outside this getter's scope.

The original Notepad v1.0.0 APK is unchanged, with SHA-256
`2c35d3dc1d41d2c761b52785c591973886fb671a2cc2e7ab047ede89599db47f`.
At 390×844, tapping its navigation button at (24,22) and advancing 100 ms makes
the original Create or edit folders entry visible partly on screen: x=-96,
width=236, y=191, height=60, with visible ancestors and alpha=1. This is an
animation frame, not verified full drawer settlement. The permanent public replay
checks visibility, viewport intersection and preservation of both exact SQLite
ID/title/body rows against a disposable copy of its saved notes.

At this checkpoint, advancing to 1000 ms reached the original drawer settlement Runnable, then stopped
at unsupported `Landroid/view/View;->requestFocus()Z`. The diagnostic retains
`Landroid/support/v4/widget/x;->a(I)V [classes.dex, PC 0x00c5]` and the upstream
queued callback locations. The copied and source note rows remain unchanged.
Full settlement and Back closure were unverified; physical native drawer input
remains unverified. Selecting Create or edit folders from the 100 ms frame reached
its Activity, then stopped during NewFolderViewHolder binding at unsupported
`Resources.getResourceEntryName(I)`. Folder creation/editing remains unverified;
this diagnostic alone does not establish the underlying binding cause.

The full local gate passes 105 Rust tests, warning-free Clippy, optimized
CLI/document replay builds and 4,096 seeded parser mutations. The optimized
public APK replay passes, including the new 100 ms drawer frame and exact-row
check, alongside the existing calculator, image/document, Activity result,
SwpieView and Notepad save/edit/Delete/Undo/restart/malformed-body checks.
The unsigned development bundle was rebuilt; its executable matches the optimized CLI
byte-for-byte. This is build evidence; no fresh physical drawer input or normal
native close is claimed. GitHub Actions remain disabled and the 50% checkpoint
remains active.

## Current-source focus and original Notepad drawer navigation

The [resource lookup checkpoint](#current-source-resource-entry-names-and-folder-diagnostic)
below supersedes this checkpoint's folder diagnostic.

Date: 2026-10-01. Explicit guest View focus ownership now completes the unchanged
Notepad v1.0.0 drawer's original settlement callback. The APK still has SHA-256
`2c35d3dc1d41d2c761b52785c591973886fb671a2cc2e7ab047ede89599db47f`.
At 390×844, navigation tap (24,22) followed by 1000 ms makes its original
Create or edit folders entry visible at x=-2, y=191, width=236, height=60,
with visible ancestors and alpha=1. Back followed by another 1000 ms removes the
drawer entry and retains the Notes screen. Both exact SQLite ID/title/body rows
remain unchanged in the disposable replay copies and the source seed.

The permanent public replay checks the 100 ms frame, 1000 ms settlement and Back
closure separately against two notes saved and edited through the original APK.
It rejects an accidentally finished Activity, verifies both titles after closure
and checks both full database rows after every step. No APK code or resource is
rewritten to perform this navigation.

The compiled FocusContract previously stopped at unsupported ViewGroup.hasFocus.
It now checks actual ownership, parent notifications, virtual focus callbacks,
listener order, rectangle identity, focusable widget defaults, literal XML flags,
before/after/block descendant policies, focused drawable state and pressed-state
clearing. It covers twelve nested groups, removal, callback GC, faults/recovery,
cycle diagnostics, missing arguments and rejected worker UI calls. Recursive
focus dispatch runs outside the large framework fallback frame; these checks
pass with the normal Rust test stack, without a larger-stack environment override.

The adapter regression separately reproduced a stale focused cell after refresh.
The shared replacement paths now notify focus loss before clearing its parent,
clear the old ownership and retain removed cells through callback GC. Compiled
checks cover both refresh and adapter removal, exact gain/loss counts, the parent
seen during loss, and collection after releasing the fixture's final reference.
This is explicit focus in the desktop non-touch profile. Automatic keyboard
traversal, touch-mode transitions, global focus observers, IME and AppKit
first-responder synchronization remain unimplemented. Physical native drawer
input remains unverified.

A fresh folder-navigation probe from the settled drawer reaches EditFoldersActivity
and stops during NewFolderViewHolder binding at unsupported
`Landroid/content/res/Resources;->getResourceEntryName(I)Ljava/lang/String;`.
The diagnostic retains the original ButterKnife and RecyclerView DEX call chain;
both copied and source note rows remain exact. Folder creation/editing remains
unverified, and this missing diagnostic method alone does not establish the
underlying binding cause.

The final local gate passes 106 Rust tests, warning-free Clippy, optimized
CLI/document replay builds and 4,096 seeded parser mutations. The optimized
public replay passes the new drawer settlement/Back checks alongside calculators,
images, grids, Activity results, documents, SwpieView, and Notepad
save/edit/Delete/Undo/restart/malformed-body checks. The unsigned development
bundle was rebuilt and matches the optimized CLI byte-for-byte (SHA-256
`6192e2a37c19765a6e0332f067efd40a92be866852c55e3383331420e40d0030`).
This is build evidence; no fresh physical drawer input or normal native close is
claimed. GitHub Actions remain disabled and the 50% checkpoint remains active.

## Current-source resource entry names and folder diagnostic

The [message/cause checkpoint](#current-source-exception-causes-and-original-folder-type-error)
below supersedes this checkpoint's missing exception constructor.

Date: 2026-10-01. Resources.getResourceEntryName now reads the name of the
requested compiled entry. It does not resolve aliases to their target's name or
require a scalar value for an ID entry or style bag. The authored Images contract
checks layout, ID, style and drawable-alias names across guest GC. Invalid IDs,
including a missing framework entry, raise a guest Resources.NotFoundException
with a message and the correct RuntimeException parent. The existing widget
regression failed at the unsupported lookup before the change and now passes.
The full framework resource-name table remains unsupported.

A fresh optimized replay of the unchanged pinned Notepad APK opens and settles
the drawer, then selects Create or edit folders. The entry-name lookup now
returns to the original ButterKnife binding code. That code next stops while
constructing its diagnostic at unsupported
`Ljava/lang/IllegalStateException;-><init>(Ljava/lang/String;Ljava/lang/Throwable;)V`.
The retained call chain includes NewFolderViewHolder and ButterKnife's
`Lbutterknife/a/c;->a(Landroid/view/View;ILjava/lang/String;Ljava/lang/Class;)Ljava/lang/Object;`
at classes.dex PC 0x0035. This is the next missing diagnostic operation, not proof
that folder binding or editing works. Both exact source and copied SQLite note
rows remain unchanged. The underlying binding cause remains to be exposed.

The full local gate again passes 106 Rust tests, Clippy, optimized CLI/document
replay builds and 4,096 seeded parser mutations. The public replay retains its
calculator, image/grid, Activity-result, document/SwpieView and Notepad
save/edit/Delete/Undo/drawer/Back/restart/malformed-body checks. The rebuilt
unsigned development bundle and optimized CLI have matching SHA-256
`2d301be9e7825fcd63017f0356d02c1dd32a38a1e40346a53af81d1eccdfe4b8`.
No fresh physical drawer or folder input is claimed. GitHub Actions remain
disabled; the 50% checkpoint is still active.

## Current-source exception causes and original folder type error

The [inflater checkpoint](#current-source-layout-inflater-factory-dispatch)
below supersedes this checkpoint's factory-dispatch scope and folder diagnostic.

Date: 2026-10-01. The message/cause constructor now retains the actual guest
references for Throwable, Exception, RuntimeException and IllegalStateException,
including null arguments. Virtual fillInStackTrace still executes with rooted
ownership. Following API-21 libcore, the fields are assigned before that callback.
The compiled Throwable contract previously failed at the missing overload and
now verifies message/cause identity across GC, callback observations, retained
cause-chain output and callback-fault cleanup. The portable constructor checks
also pass on desktop Java 17; its callback assignment order differs, so that
observation check is explicitly API-21-only. Cause-only constructors, initCause,
suppression and StackTraceElement arrays remain unsupported.

The unchanged pinned Notepad APK now reaches its original ButterKnife throw at
classes.dex PC 0x0038. The guest IllegalStateException reports:
`View 'left_button' with ID 2131493021 for field 'leftButton' was of the wrong type.`
Both copied and source SQLite rows remain exact. This is failure evidence;
folder binding, creation and editing remain unverified.

A standalone diagnostic built against DROIDLESS's own parser reads the original
APK's view_new_folder binary XML and confirms the left_button tag is ImageButton.
The tagged [NewFolderViewHolder](https://github.com/MohMah/android-notepad/blob/v1.0.0/app/src/main/java/ir/cafebazaar/notepad/activities/editfolders/NewFolderViewHolder.java)
declares AppCompatImageButton. Current source retains LayoutInflater factories
but constructs XML widgets directly without invoking them. The next shared fix
is factory callback dispatch; changing the APK or aliasing the widget's class
would bypass the guest creation path.

The full local gate passes 106 Rust tests, Clippy, optimized CLI/document replay
builds and 4,096 seeded parser mutations. The unchanged public APK replay passes,
including the drawer's 100 ms frame, 1000 ms settlement and Back closure with
exact retained rows. The rebuilt unsigned bundle matches the optimized CLI
(SHA-256 `b32148b7cf78b40b4c819a5cc174b45fd3e55254b4b41bb64a531575d42dc81e`).
No fresh native folder interaction is claimed. GitHub Actions remain disabled
and the 50% checkpoint remains active.

## Current-source LayoutInflater factory dispatch

Date: 2026-10-02. Shared binary XML inflation now calls the installed guest
Factory/Factory2 before default construction. The actual inflater instance flows
through Activity content, include/merge children and native ViewStub replacement.
One managed AttributeSet reaches the callback and fallback constructor. A factory
View is adopted directly without calling its constructor twice. Factories receive
the supplied parent even when the root is detached.

LayoutInflater.from resolves virtual Context.getSystemService. cloneInContext
copies factories and resets one-setter state. New clone factories precede inherited
callbacks, with null fallback; Factory2 retains inflation priority when a clone
sets only Factory. Native ViewStub captures a clone and supports an explicit
replacement inflater, whose own context controls construction. Private factories,
filters, theme wrapping and full custom-inflater subclass behavior remain outside
the profile. These semantics follow the
[API-21 reference](https://android.googlesource.com/platform/frameworks/base/+/android-5.0.0_r1/core/java/android/view/LayoutInflater.java).

The authored compiled InflaterContract checks service identity and virtual routing,
missing services, null/repeated setters, factory identities and dispatch overloads,
widget substitution, include/merge children, detached parent parameters, constructor
and onFinishInflate counts, real compiled attribute IDs and callback/constructor
attribute identity. It also checks cloned and explicitly assigned ViewStubs,
merged callback order/context, callback GC, propagated faults and recovery,
rejected worker UI access and collection of released temporary roots. A 64-layer
merged factory chain inflates on the normal test stack; a 65th merge fails with
the documented bound. The baseline failed at the missing layout-inflater service.
The expanded check also caught and corrected ViewStub using its own context
instead of the explicitly assigned inflater's context.

The new callback path exposed shared constructor dependencies in the original
APKs. View listener queries, bounded monitored Hashtable, canonical-loader
loadClass, Class.asSubclass, StateSet prefix trimming and Java float rounding
now pass compiled regressions. Portable Hashtable, reflection and Math contracts
also pass on desktop Java 17. AppCompat checks add default untransformed text,
retained checkmark/hint/link palette identity, editor-action callback delivery,
bounded live ColorStateList arrays and seven-field Configuration copying/equality.
Configuration uses the declaring-owner keys read by DEX; signed-zero and API-21
NaN comparisons are checked. Palette/checkmark state is retained without claiming
painting, and editor callbacks do not imply host Return or IME navigation.

The unchanged pinned Notepad APK now opens Edit Folders and binds the original
folder editor and button listener. Its own AppCompat Factory2 supplies the actual
AppCompatImageButton expected by ButterKnife. No APK patch or class alias is used.
A copied-data headless run returns exit zero and retains both exact note rows;
the immutable original data is also unchanged. A separate Back replay returns to
Notes with both exact rows and labels retained. The permanent compatibility suite
now checks folder editor/button binding, Back and row retention on copied data.
The former wrong-type error is
resolved. Starting creation through the original left-button callback instead
reaches RecyclerView.requestChildFocus and rejects the missing inherited
ViewGroup.offsetDescendantRectToMyCoords(View, Rect) bridge. That failed run also
preserves both exact rows. Creation/editing and fresh native folder input remain
unverified.

The final full local gate passes 109 Rust tests, Clippy with warnings rejected,
optimized workspace and document-replay builds, and 4,096 seeded parser mutations
without panics. A fresh optimized folder-opening replay returns exit zero and
preserves both exact note rows. The rebuilt unsigned development bundle matches
the optimized CLI byte for byte (SHA-256 `eaf53c3de5af2748495e2c5d87e37435bff12bfd712f214dfb9afb892f243082`). No fresh native
folder interaction or public-release update is claimed. GitHub Actions remain
disabled, and the 50% checkpoint remains active.

The full optimized compatibility replay passes unchanged public calculators,
SwpieView folder thumbnails/full-screen viewing/Back/gestures and the existing
slideshow diagnosis. Notepad save/restart, existing-note edit, Delete, timed
feedback/dismissal, Undo/restart, drawer frame/settlement/Back and malformed-body
preservation still pass. The new original Edit Folders editor/listener binding
and Back checks pass with both exact note rows retained on copied data. The
optimized creation probe confirms the same missing coordinate bridge and exact
row retention; creation/editing remain unverified.

## Current-source descendant coordinates and resource backgrounds

Date: 2026-10-02. Shared ViewGroup descendant Rect conversion now follows actual
managed parents, layout positions and retained scroll offsets in both directions.
Java wrapping arithmetic preserves inverted/empty bounds and overflowing edges.
Self conversion is a no-op; invalid ancestry retains preceding translations before
raising the guest fault. Cyclic/overlong chains are bounded at 128 steps. Rect
width/height use signed wrapping subtraction. The compiled CoordinateContract
checks nested groups, both directions, self/null, overflow, invalid ancestry and
GC; its runtime check also covers stored scroll offsets, cycles and recovery.
This does not implement scroll setters, matrix transforms or clipping.

View.setBackgroundResource loads through the actual Context and dispatches the
virtual Resources.getDrawable(id, theme) and View background callbacks. Nonzero
repeated resource IDs retain Drawable identity, while changing the Drawable or
flat background color clears the cache. Zero clears through the actual callback;
resource IDs are retained only after success. BackgroundResourceContract checks
lookup/cache identity, real Resources/View overrides, callback GC, faults and
recovery. Untinted background list/mode queries return null. Tint setters/XML tint
application, native tint painting and full background coherence remain unsupported.

The unchanged pinned Notepad APK now passes the former child-coordinate bridge,
Rect dimensions, resource background and untinted queries when its original folder
left-button listener runs. It next rejects Log.e(String, String, Throwable) while
that listener reports a keyboard-service error. A copied-data replay retains both
exact seed/copy note rows and writes no Folder row. The permanent compatibility
suite diagnoses this exact callback and verifies those data boundaries; it marks
folder creation false. Opening Edit Folders and returning through Back remain
separate successful workflows. Folder creation/editing, software keyboard support
and fresh native folder input remain unverified. No APK patch or class alias is
used, and the original APK SHA-256 remains
`2c35d3dc1d41d2c761b52785c591973886fb671a2cc2e7ab047ede89599db47f`.

The full local gate passes 111 Rust tests, Clippy with warnings rejected,
optimized workspace/document-replay builds and 4,096 seeded parser mutations
without panics. The full optimized compatibility replay passes the unchanged
public calculators, SwpieView folder/viewer/Back/gestures and slideshow diagnosis,
plus Notepad save/restart, existing-note edit, Delete, timed feedback, Undo,
drawer settlement/Back, folder opening/Back and malformed-body preservation.
The new folder-creation diagnosis passes with exact original/copy rows retained
and no Folder row written. A separate fresh optimized copied-data probe confirms
the same next logging gap and unchanged immutable seed.

The rebuilt unsigned development bundle matches the optimized CLI byte for byte
(SHA-256 `15a69674f8f78b72ac5402b83a1270745f0e0da8275199fb4b6c427121856c73`). No fresh
native folder interaction or public-release update is claimed. GitHub Actions
remain disabled, and the 50% checkpoint remains active.

## Current-source hardware keyboard requests and folder persistence

Date: 2026-10-02. Log.d/i/w/e message/Throwable overloads now use the existing
bounded retained-DEX renderer. Guest description/cause callbacks retain roots
through GC; their faults propagate and clean up. Compiled checks cover all four
levels, null arguments, circular causes, GC and fault recovery. The baseline
failed at the missing three-argument logging overload. Host stderr returns zero;
Android log-buffer byte counts and full stack-formatting parity are not claimed.

Context's canonical input_method service now models a hardware-keyboard runtime
with no served software IME. The two-argument show/hide methods return false,
validate non-null targets/flags and preserve text and focus. The compiled contract
checks cached identity across Contexts and GC, detached/attached Views, nulls,
window tokens and focus/text retention. It previously received a null service.
Software keyboards, InputConnection and ResultReceiver overloads remain unsupported.

Base touch release now invokes virtual requestFocus for enabled, clickable Views
that are focusable in touch mode and do not already own focus. Successful focus
suppresses that first tap's click; a later tap dispatches the real click. EditText
has the clickable default, with explicit overrides retained. The compiled touch
contract checks cancellation, focus/click ordering, disabled/non-touch-focusable
Views and callback GC. The baseline failed its default-clickability check.
Automatic traversal and AppKit first-responder synchronization remain ahead.

The unchanged pinned Notepad APK now accepts an editor tap through its original
focus listener. Input followed by the original Done-button callback persists
exactly one Folder row named Runtime folder with ID 1. Both immutable seed and
copied-data note rows retain their exact IDs, titles and bodies. Rendering that
new row then fails at android.text.TextPaint through TextInputLayout. The permanent
compatibility replay diagnoses this dependency and checks the exact folder/notes
using new SQLite connections after the process exits. It keeps complete folder
creation false and records persistence before layout failure separately.
A plus-button replay exits cleanly but writes no folder; it is not evidence of
successful creation. Complete folder creation/editing, row display/restart and
fresh native folder input remain unverified. No APK patch or class alias is used.

The full local gate passes 114 Rust tests, Clippy with warnings rejected,
optimized workspace/document-replay builds and 4,096 seeded parser mutations.
The full optimized public replay still passes both calculators, SwpieView
folder/viewer/Back/gestures and its slideshow diagnosis, plus Notepad save/restart,
existing-note edit, Delete, timed feedback, Undo, drawer settlement/Back,
empty-folder opening/Back and malformed-body preservation. The new folder-row
persistence diagnosis passes. A separate optimized original-APK probe confirms
ID 1/Runtime folder, the same TextPaint fault and both exact immutable/copied note
rows. The rebuilt unsigned bundle matches the optimized CLI byte for byte
(SHA-256 `fb7a2e5723fd837e959ff160e24d7a1039057b87a26eb4f8edc6e64be92cfbd4`).
No new native folder interaction or public-release update is claimed. GitHub
Actions remain disabled; the 50% checkpoint remains active.

## Current-source TextPaint and child drawable states

Verified on macOS ARM64 on 2026-10-02; this extends current source after the
hardware-keyboard/folder-persistence checkpoint. The v0.1.0 release predates it.

Paint/TextPaint default and flag constructors now retain API-21 black color,
0x500 base flags and TextPaint density 1.0. Inherited Paint methods and canonical
public fields retain exact state and drawableState array identity through GC.
The compiled baseline failed at missing TextPaint. Copy constructors, text/font
metrics, shaping and native Canvas text painting remain outside this increment.

ViewGroup child-state mode now refreshes actual guest callbacks, sizes and merges
child arrays including reserved zero slots, and propagates checked-child changes.
Duplicate-parent children refresh with their group; combining duplication and
aggregation raises IllegalStateException. Native snapshots and the result array
remain rooted through callback GC and fault recovery. The compiled baseline
failed at the missing child-state query. Hostile cycle/deep-tree checks exposed
excessive stack use in the large framework fallback; recursive state callbacks
now use the same small dispatch path as focus. Cycles are rejected, with 16
active query and 32 synchronous-call ceilings. Android state caching and complete
selection/window states remain unsupported. The existing touch fixture now
expects the duplicate-parent child's additional refresh notification; both
compiled touch checks pass with the corrected exact notification counts.

The unchanged pinned Notepad APK now passes TextPaint and child-state setup while
constructing its saved folder row. Its original editor focus/Done callbacks still
write exactly one Folder row: ID 1, Runtime folder. Rendering then stops in
TextInputLayout.setErrorEnabled at unsupported error-label text appearance
color/theme values. Both immutable seed and copied-data notes retain exact IDs,
titles and bodies. The permanent optimized replay and a separate fresh-data
optimized probe confirm the row, failure location and retained notes through new
SQLite connections. No APK patch or class alias is used. Complete folder
creation/editing, saved-row display/restart and fresh native folder input remain
unverified; folder creation stays false in the catalog.

The full local gate passes 116 Rust tests, Clippy with warnings rejected,
optimized workspace/document-replay builds and 4,096 seeded parser mutations.
The complete optimized public replay also passes both calculators, SwpieView
folder/viewer/Back/gestures and its slideshow diagnosis, plus Notepad save/restart,
existing-note edit, Delete, timed feedback, Undo, drawer settlement/Back,
empty-folder opening/Back, the new folder appearance diagnosis and malformed-body
preservation. The rebuilt unsigned development bundle matches the optimized CLI
byte for byte (SHA-256 `73da902041deb1674cdf422590e93b162cd4f62f2c73c6ba2cccee63e07b9e86`). No new native folder interaction or
public-release update is claimed. GitHub Actions remain disabled, and the 50%
checkpoint remains active.

## Current-source themed folder creation and restart

Date: 2026-10-02. Current source, beyond the published v0.1.0 archive.

Typed color calls now invoke the guest Context's theme getter and copy the style
snapshot into each TypedArray. Compiled checks cover inline colors, ordered
selectors, resource/theme aliases, null/default values, theme changes after array
creation, invalid indices, callback GC and fault cleanup. TextView appearance
uses the supplied Context, invokes the actual color setter and changes pixel size
with layout invalidation only when needed. The original API-21 unresolved theme
value raises RuntimeException, so Notepad executes its own caught fallback style.
Hint/link/shadow appearance, fonts, dynamic native state colors and full styled
array recycle parity remain outside this profile.

The first complete public replay exposed a SwpieView vector setup dependency on
framework color/white. The shared bounded lookup now resolves the fixed API-21
white, black and transparent values, including APK aliases. The compiled color
contract checks typed colors/lists and Resources getters against those exact
ARGB values. Zero getColor IDs and unknown framework colors remain explicit
failures. The unchanged SwpieView document replay again decodes three images.

Sized ViewGroup attachment invokes the virtual default factory, retains the same
layout-parameter object, sets canonical width/height fields and invokes virtual
indexed addView with index -1. Compiled checks cover default FrameLayout,
LinearLayout and table parameter types/values, orientation, guest overrides,
callback GC, null factories, thrown callbacks, recovery and root cleanup. The
baseline failed at the missing LinearLayout default factory; the completed
contract and existing touch/inflater/widget checks pass.

The pinned unchanged Notepad APK (SHA-256
`2c35d3dc1d41d2c761b52785c591973886fb671a2cc2e7ab047ede89599db47f`)
now completes its original folder editor/Done callbacks and renders exactly one
visible row, ID 1/Runtime folder. A separate fresh-data optimized probe creates
the folder, opens its saved row in a fresh process, then opens it and returns
through Back in another fresh process. All three exit zero. New SQLite
connections confirm exactly one Folder and preserve both immutable/copied note
IDs, titles and bodies after each phase. The permanent public replay also checks
the saved row's visibility and viewport bounds, restart, Back and exact rows.
No APK patch or class alias is used. Folder editing/deletion and fresh physical
folder input remain unverified; creation and saved-row restart/Back are now true
in the catalog. This does not establish Android-wide styling or UI parity.

The final local gate passes 119 Rust tests, Clippy with warnings rejected,
optimized workspace/document-replay builds and 4,096 seeded parser mutations.
The complete optimized public replay covers both calculators, image/grid/result
fixtures, SwpieView folder/viewer/Back/gestures and its slideshow diagnosis, plus
Notepad save/restart, existing-note editing, Delete, timed feedback, Undo, drawer
settlement/Back, empty-folder opening/Back, saved-folder creation/restart/Back and
malformed-body preservation. The refreshed unsigned development bundle matches
the optimized CLI byte for byte (SHA-256 `9aacb1099a41e08ec5246341d6ffd477438b612040d8f0e36c5bf4f57cd51ec4`).
No new physical folder interaction or public release is claimed. GitHub Actions
remain disabled, and the 50% checkpoint remains active.

## Current-source saved-folder binding and focus

Date: 2026-10-02. Current source, beyond the published v0.1.0 archive.

The old convenience attachment path bypassed the APK's indexed addView override.
It made the saved row appear while leaving its editor below the parent bounds.
Child-only and child/index overloads now invoke the actual guest getter, default
factory and indexed attachment callbacks. TextInputLayout binds its own EditText;
the observed editor now occupies y=116..156 within its y=100..156 parent.
Compiled checks cover parameter identity/order, virtual callbacks, null defaults,
GC, thrown callbacks and root cleanup.

Typeface state retains cached sans/serif/monospace identities and four styles.
Paint and TextView retain nullable faces; View family/style reaches the native
button, label and editor fonts. The local AppKit component check validates all
36 control/family/style combinations. TextWatcher delivery uses real guest
before/on/protected/after callbacks, UTF-16 deltas, fresh setText buffers and
retained native-input/append buffers. Compiled DEX checks include reentrant after
edits, detached old buffers, host input, GC and callback faults. Scalar
ValueAnimator float/int keyframes reuse the existing host clock and callback
roots; initial updates, delayed starts, cancellation/end, values and failure
recovery pass the compiled contract. ARGB channels/packing and retained Paint
shadow state now support the actual collapsing-label callbacks. Asset fonts,
Android font metrics, Canvas shadow rasterization and complete animation APIs
remain unsupported or unverified.

The unchanged SHA-256-pinned Notepad APK still creates one visible folder and
reopens it after restart/Back. A root tap now focuses its saved-row editor and
host input displays a pending name with exit zero. Both note IDs/titles/bodies
and the saved Folder ID/name remain exact. A fresh process discards that
unconfirmed input and displays the original saved name. This is headless input
proof; physical native folder input remains unverified.

The original right-button rename confirmation updates the same Folder ID/name
in SQLite, then fails in EditFolderViewHolder.u at DEX PC 0x0018 because the
collapsing label calls unsupported TextPaint.ascent()F. A separate data copy
preserves both exact note rows. The optimized public replay checks this precise
partial-write diagnosis; rename and folder deletion are not marked complete.
No APK patch, class alias or replacement callback is used.

The final local gate passes 125 Rust tests, Clippy with warnings rejected,
optimized workspace/document-replay builds, 4,096 parser mutations and the native
font component check. The expanded optimized public replay preserves the
calculator/image/grid/result and SwpieView workflows, all existing Notepad note,
Delete/Undo/drawer/folder checks, and adds saved-row focus/input, restart discard
and the isolated rename diagnosis. The unsigned development bundle matches the
optimized CLI byte for byte (SHA-256 `8f7003d36e3c415cd1fc3b991fb1ce454cb673665c1e83a527ea50da0d10ffb2`). No new public release or
physical folder interaction is claimed. GitHub Actions remain disabled; the
50% checkpoint stays active.

## Current-source folder rename and host font metrics

Date: 2026-10-02. Source commit b6701fc, beyond the published v0.1.0 archive.

The baseline reproduced the original Notepad rename's partial write: Folder ID 1
received the new name, then EditFolderViewHolder.u failed at DEX PC 0x0018 on
TextPaint.ascent()F. Paint/TextPaint now retain the API-21 default size of 12,
ignore negative sizes and support zero. Ascent/descent come from the same actual
AppKit font selection used by native buttons, labels and editors. Family/style,
finite size and the 4096-pixel ceiling are checked on both sides of the scalar
FFI. This is host-font measurement, not Android font or pixel parity.

The compiled FontMetricsContract checks three families and four styles, retained
size/face across GC, size scaling, nullable faces, ignored negative sizes and
zero metrics. The native component check compares the metric outputs with the
actual fonts assigned to all 36 control/family/style combinations and rejects
invalid family/style/size inputs. Font measurement currently requires macOS;
asset fonts, shaping and Android baseline parity remain unsupported.

The unchanged Notepad APK retains SHA-256
`2c35d3dc1d41d2c761b52785c591973886fb671a2cc2e7ab047ede89599db47f`.
A fresh-data optimized probe completes six separate processes: create, saved-row
restart, Back, original rename confirmation, renamed-row restart and Back.
Each exits zero. The visible row is checked inside the viewport; new SQLite
connections confirm the same folder ID/new name and preserve both seed/copy note
IDs, titles and bodies exactly at every phase. The permanent public replay now
checks rename confirmation/restart/Back alongside focus and unconfirmed-input
discard. No APK patch, class alias or replacement callback is used. Folder
deletion and physical native folder input remain unverified.

The local gate passes 126 Rust tests, Clippy with warnings rejected, optimized
workspace/document-replay builds, 4,096 seeded parser mutations and the native
font component check. The full optimized public replay also passes both
calculators, image/grid/result fixtures, SwpieView folder/viewer/Back/gesture
checks and its slideshow diagnosis, plus every existing Notepad save/edit,
Delete/Undo/timed feedback, drawer/folder and malformed-body preservation check.
The refreshed unsigned development bundle matches the optimized CLI byte for
byte (SHA-256 `074b87a0c697295efbf74e0af07dc340b52477cc67384f0bd100b7b4b41c5153`).
No new public release or physical folder interaction is claimed. GitHub Actions
remain disabled; the 50% checkpoint remains active.

## Current-source native editor focus bridge

Date: 2026-10-02. Source commit a3b68c0, beyond the published v0.1.0 archive.

The native component baseline created an actual NSTextView field editor without
delivering guest focus. Editable AppKit fields now report successful
becomeFirstResponder transitions through Runtime.focus. That dispatcher checks
the main-thread View handle and enabled/visible state, invokes virtual guest
requestFocus, drains navigation and collects. Refused focus aborts native editing;
callback errors stop the host. Native tree updates suppress new focus requests
to avoid reentering Rust while controls are replaced. The existing AppKit text
selection path is retained.

HostFocusContract executes real DEX editor callbacks, including GC, repeated
focus without another gain, transfers, disabled/hidden/non-focusable targets,
fault state/recovery, invalid handles and released roots. The actual AppKit
component check exercises first-responder gains, native key-view traversal,
selection through redraw, draw-time refusal, rejected guest focus and failed
callbacks. These are component checks; physical mouse/keyboard interaction in
the public folder screen remains unverified. Reverse guest-to-AppKit updates,
focus loss when leaving all editors, Android traversal/IME and complete input
parity remain ahead.

The CLI --focus-at INDEX shares the native dispatcher and reuses --input-at's
enabled/visible editor lookup. A fresh-data optimized probe uses it to create
and rename a folder in the unchanged SHA-256-pinned Notepad APK. Six processes
complete creation, saved-row restart, Back, rename confirmation, renamed-row
restart and Back with exit zero. New SQLite connections preserve the same
folder ID/new name and both seed/copy note IDs, titles and bodies exactly at
every phase. The permanent replay also exercises host-focused pending input,
restart discard and host-focused original rename confirmation. No APK patch or
replacement callback is used.

The local gate passes 127 Rust tests, Clippy with warnings rejected, optimized
workspace/document-replay builds, 4,096 seeded parser mutations and native
font/focus component checks. The refreshed unsigned development bundle matches
the optimized CLI byte for byte (SHA-256
`05c92947539d76a01a99e7562f52827b1523c5c58eeea5bbb9329ce054337350`).
The complete optimized public replay passes both calculators, image/grid/result
fixtures, SwpieView folder/viewer/Back/gesture checks and its slideshow diagnosis,
plus all Notepad note editing, Delete/Undo/timed feedback, drawer/folder,
host-focus pending input/discard, rename/restart/Back and malformed-body checks.

Fresh native Notepad windows were confirmed live and visible, including a
390x600 logical viewport whose frame was {{445,149},{390,632}}. The UI controller
still returned cgWindowNotFound for the exact bundle. Those probe processes were
terminated with status 143 after attachment failed; this is not clean native
closure or physical interaction proof. Folder deletion and fresh physical
folder input remain unverified. GitHub Actions remain disabled, no new public
release is claimed and the 50% checkpoint stays active.


## Current-source themed contexts and folder-delete dialog boundary

Date: 2026-10-02. Runtime source commit 5ca50c5, beyond the published v0.1.0 archive.

The original focused saved-row delete listener previously stopped in the bundled
AppCompat builder on unsupported ContextThemeWrapper. The framework now supports
an independent lazy theme copy, virtual theme/resource/service callbacks, cached
Resources and a cloned inflater retaining the base factories and wrapper context.
New themes start empty; setTo copies style state while preserving destination
ownership. Compiled contracts check immutable typed arrays, theme isolation,
default API-21 theme IDs, late attachment, null/type errors, callback GC, fault
recovery and the 32-call native wrapper bound. Complete system themes,
configuration overrides and automatic XML theme wrapping remain unsupported.

The unmodified SHA-256-pinned Notepad APK now passes that builder setup. Focusing
saved editor 1 and tapping its left button reaches the actual clickLeftButton
callback at DEX PC 0x0057, then the bundled support Dialog constructor at PC
0x0013. It fails explicitly at `Landroid/app/Dialog;-><init>(Landroid/content/Context;I)V`
with exit status 1. New SQLite connections confirm that the same folder and both
exact note IDs, titles and bodies survive in the disposable copy and seed. The
permanent replay checks this boundary and leaves folder deletion marked false.
The confirmation window, Cancel, confirmed deletion and native folder input
remain unverified; no replacement dialog or APK patch is used.

The final local gate passes 129 Rust tests, warning-free Clippy, optimized
workspace/document-replay builds, 4,096 seeded parser mutations and native
font/focus component checks. The full optimized public replay retains all prior
calculator, image/grid/result, SwpieView and Notepad workflow checks and adds the
folder-delete boundary diagnosis. It records the CLI digest with verification
false at startup and certifies it only after the full suite succeeds and the
final digest matches. This rejects a binary changed by a concurrent local build.
The unsigned development app bundle matches the optimized CLI byte for byte:
SHA-256 `4c045e8af412f71b4942c1ee6e75a408726bfd19d3336c4554ae0212e5e0b8ab`.
GitHub Actions stay disabled. No new public release or physical interaction is
claimed, and the 50% checkpoint remains active.


## Current-source owned dialog windows and state-list backgrounds

Date: 2026-10-02. Runtime source commit ebec998, beyond the published v0.1.0 archive.

Dialog construction now retains its own themed context, Window, decor and content
container. Actual virtual theme/service/inflater calls and installed factories run
through the wrapper. Content replacement retains decor identity without replacing
the Activity root. Window attributes retain identity and invoke real guest
callbacks. Dialog.create runs onCreate(null) once after a successful return; a
failed creation can retry. Compiled contracts check ownership, feature/layout/
flag/title state, content replacement, callback GC, faults, invalid arguments,
unregistered-object collection and bounded recursion. Native dialog presentation
and input remain unsupported.

Theme.resolveAttribute now fills TypedValue for supported values, attribute and
resource aliases and style references. Missing/cyclic values leave output intact;
unresolved references keep resourceId zero. Resolution is capped at 20 steps.
View.EMPTY_STATE_SET has one inherited empty-array identity while APK-declared
fields retain their own identity. StateListDrawable selects ordered positive,
negative, zero-terminated and wildcard specifications, retains arrays, invokes
actual guest/child state callbacks and measures the current child. Selected color
leaves render through nested backgrounds. Shared pressed/enabled setters and
normal touch down/up/cancel refresh state. Tests check rendered colors, equal-state
identity, GC/fault recovery, malformed/oversized inputs, cycles and explicit failure
for unsupported selected leaf painting. General composite painting and selector
XML inflation remain unsupported.

The unmodified SHA-256-pinned Notepad APK passes drawer measurement and the actual
Dialog constructor. Focusing saved editor 1 and tapping its left button reaches
EditFolderViewHolder.clickLeftButton at DEX PC 0x0057, then stops explicitly at
`Landroid/support/v7/a/q;->setCancelable(Z)V` with exit status 1. The bundled builder
reaches that call at PC 0x00d2. New SQLite connections verify that the seed and
disposable copy retain the exact folder ID/name and both note IDs/titles/bodies.
The permanent replay certifies construction and this new boundary while keeping
folder deletion false. Confirmation, Cancel and physical folder input remain
unverified.

The complete local gate passes 132 Rust tests, warning-free Clippy, optimized
workspace/document-replay builds, 4,096 seeded parser mutations and native
font/focus component checks. The complete public replay preserves prior calculator,
image/grid/result, SwpieView and Notepad creation/rename/restart/Back workflows and
certifies one unchanged optimized CLI digest. The unsigned development bundle
matches that CLI byte for byte: SHA-256 `eeeedb82092aab345f66202e1f6bfe0c4696ec340180b56be4940c9a5d182a66`.
GitHub Actions remain disabled. No new public release or physical dialog
interaction is claimed; the 50% checkpoint remains active.


## Current-source weak references and dialog cancellation messages

Date: 2026-10-02. Runtime source commits ff681c7 and 009ff4c, beyond the published
v0.1.0 archive.

WeakReference now contributes a weak heap edge. Collection clears unreachable
referents, including cyclic targets, while retaining a reference object's other
strong fields and honoring independent roots. Calls through Reference and APK
subclasses use the same behavior. clear(), nonreused handles and invalid receiver/
target/queue arguments have a compiled DEX contract. Non-null ReferenceQueue
registration and enqueue/finalization APIs remain unsupported; WeakHashMap retains
its explicit strong-key profile.

Dialog supports cancellation flags, outside-close settings and Back policy in the
main-thread unshown profile. Registered cancel/dismiss/show/key listener objects
are type-checked and retained. cancel posts a fresh message once, then executes
the actual virtual dismiss override. Unshown dismissal follows the SDK no-op.
The listener handler holds the Dialog weakly, so a pending message preserves its
payload without retaining an abandoned owner. Delivery executes the real guest
interface callback through the existing Handler/Looper queue and can receive null
after owner collection. Message.obtain snapshots supported scalar/obj/target/
callback fields without copying queue-use state or delivery time; sendToTarget
uses the actual Handler. Compiled tests check listener replacement snapshots,
queued-message cloning, duplicate sends, custom cancellation messages, callback
GC/fault cleanup, owner collection and invalid inputs without accidental enqueue.
Visible dialog presentation, dismissal and corresponding lifecycle events remain
unsupported. Registration alone is not shown-dialog evidence.

The original SHA-256-pinned Notepad APK now completes its bundled dialog builder,
cancellation flags and listener setup. Focusing saved editor 1 and tapping the
left button reaches EditFolderViewHolder.clickLeftButton at DEX PC 0x0057, then
its builder at PC 0x0004. It fails explicitly at
`Landroid/support/v7/a/q;->show()V` with exit status 1. Fresh SQLite reads verify
that the disposable copy and seed retain the exact folder ID/name and both note
IDs/titles/bodies. The permanent public replay certifies builder setup and this
presentation boundary while keeping confirmed deletion false. Confirmation,
Cancel through native input and physical folder input remain unverified.

The complete local gate passes 135 Rust tests, warning-free Clippy, optimized
workspace/document-replay builds, 4,096 seeded parser mutations and native
font/focus component checks. The complete public replay retains prior calculator,
image/grid/result, SwpieView and Notepad create/rename/restart/Back workflows and
certifies one unchanged CLI. Its unsigned development bundle matches byte for
byte: SHA-256 `53abe7906a87048e2de8f6857c36ef90d2895f8f12170327deb7738b9f8da309`. GitHub Actions stay disabled.
No new public release or physical dialog interaction is claimed. The 50%
checkpoint remains active.


## Current-source modal dialog surfaces and AppCompat content boundary

Date: 2026-10-02. Runtime source commit bb23177, beyond the published v0.1.0 archive.

The runtime now retains each attached Dialog independently of the Activity root
and renders visible surfaces into separate real AppKit NSPanels. The authored
DialogSurfaceContract runs actual DEX creation/start/attachment/detachment/stop
methods and queued show/cancel/dismiss listeners. It checks stable Activity state,
distinct window tokens, hide/reopen behavior, nested Back, cancellation policy,
outside touch, actual button callbacks, blocked background input, text/focus,
window dimension bounds, host surface identity, collection after dropping guest
statics, and recovery from create/start/stop failures. Existing unshown-window
and weak cancellation contracts continue to pass.

The native component check creates real panels and native button controls, sends
their control callbacks, checks parent/child ownership and close routing, blocks
lower-panel controls, reuses/resizes a panel, restores a previous panel and retires
both without closing the Activity. It is a component test, not physical input into
a public APK. Native frame application suppresses focus/resign callbacks to avoid
reentering Rust while applying controls; the host reports the actual key surface.

The original SHA-256-pinned Notepad APK now enters
`Landroid/support/v7/a/q;->onCreate(Landroid/os/Bundle;)V` at PC 0x0012, then
bundled AppCompat content installation. The original focused folder-delete
listener still calls at PC 0x0057; its builder calls at PC 0x0004. The new first
failure is `Landroid/widget/FrameLayout;->setForeground(Landroid/graphics/drawable/Drawable;)V`
from `Landroid/support/v7/a/ac;->n()V` at PC 0x01b1. The unchanged APK digest is
`2c35d3dc1d41d2c761b52785c591973886fb671a2cc2e7ab047ede89599db47f`.
The diagnostic preserves the same saved folder ID/name and both exact note
IDs/titles/bodies in the disposable copy and original seed. The public dialog is
not yet displayed; confirmation, Cancel through native input and confirmed folder
deletion remain unverified. No APK patch or replacement confirmation is used.

The source gate covers 136 Rust tests, warning-free Clippy, optimized workspace
and document-replay builds, 4,096 seeded parser mutations, and native font,
focus and panel components. GitHub Actions remain disabled. No new public release
or physical dialog interaction is claimed. The 50% checkpoint remains active.


The complete public replay passes prior calculator, image/grid/result, SwpieView
and Notepad create/rename/restart/Back workflows and certifies the new onCreate/
foreground boundary against one unchanged release CLI. Its unsigned development
bundle is byte-identical: SHA-256 `9e81baab02d20dbef827f614c43ddd0da57a55b31c9f6ce9f8f527ec469b1697`. The replay keeps confirmed
folder deletion false; entering onCreate is not completed dialog creation or
presentation. The live site mirrors these source documents and compatibility data.


## Current-source foreground overlays and dialog layout boundary

Date: 2026-10-02. Runtime source commit 7c84ff2, beyond the published v0.1.0 archive.

FrameLayout now retains actual programmatic foreground Drawable identity and
paints supported color/selector leaves above its native descendants. A separate
AppKit overlay preserves native child control hit testing and combines drawable
ARGB with ancestor alpha. Replacement/null clears the old weak callback and
retires the overlay. Visibility/color invalidation, View-driven selector state,
constant selector padding, virtual getPadding and setBounds share the managed
callback and layout paths. Effective padding takes each edge's maximum without
rewriting the user's padding getters. Other gravity can be stored for a null
foreground; painting outside FILL fails explicitly. Theme/XML foregrounds,
bitmap/composite painting and tint/Canvas parity remain outside this increment.

The actual compiled ForegroundContract checks original getter identity, no-op
same assignment, click-through guest delivery, recoloring, visibility, pressed
selector state, per-edge padding, replacement/clearing, weak callback collection,
invalid references, unsupported paint and recovery after guest padding faults
with System.gc during callbacks. The native component creates a real NSButton
beneath the overlay and checks host hit testing, stacking, ARGB/ancestor alpha,
resize/reuse and retirement. It is a component test, not physical input into the
public Notepad dialog.

The original SHA-256-pinned Notepad APK passes the prior setForeground boundary.
Its actual focused EditFolderViewHolder.clickLeftButton still calls at PC 0x0057,
its builder at PC 0x0004 and its bundled dialog onCreate at PC 0x0012. The new
first failure is `resource @0x00000000 missing or complex`, while the bundled
`Landroid/support/v7/a/ac;->b(I)V` installs dialog content at PC 0x0017 through
LayoutInflater.inflate. The unchanged APK SHA-256 is
`2c35d3dc1d41d2c761b52785c591973886fb671a2cc2e7ab047ede89599db47f`.
The disposable attempt preserves the saved folder ID/name and both exact note
IDs/titles/bodies, and leaves the seed intact. Dialog creation/presentation is
still incomplete; public confirmation, Cancel and confirmed folder deletion
remain unverified. No APK patch or replacement confirmation is used.

The complete local gate passes 137 Rust tests, warning-free Clippy, optimized
workspace/document-replay builds, 4,096 seeded parser mutations and native
font/focus/dialog/foreground component checks. GitHub Actions remain disabled.
The unchanged optimized CLI and its unsigned development bundle are byte-identical:
SHA-256 `71ce086381c8a340f6cf22b906b4de69dd9e6f5736347e3e0d537a5b53d25628`. The complete public replay passes calculator, image/grid/result, SwpieView and
Notepad create/edit/delete/Undo/drawer/folder/restart workflows, and certifies
this same unchanged CLI. Its folder-delete diagnostic records the new resource
boundary while keeping confirmed folder deletion false. The 50% checkpoint remains active; no new release or physical
public dialog interaction is claimed.

## Current-source styled resources and dialog inflation

Date: 2026-10-02. Source increments: cfb5d75 (default/XML styles), c9eff8a
(null XML images), eea2b6f (typed floats) and 3513729
(typed-array output), plus 6c3b9bf (inherited Material defaults). All are beyond the published v0.1.0 archive.

Context and Theme styled-attribute calls now select defStyleAttr when it resolves
to a style, otherwise defStyleRes, then overlay XML style and explicit XML values
above theme defaults. An explicit null style suppresses the fallback. Existing
TypedArrays retain their theme snapshot through later changes. Resources.obtainAttributes
reads explicit XML values. Compiled DEX checks cover precedence, alias/cycle
handling, getter callbacks, GC, invalid receivers and recovery. The precedence
was checked against [API-21 resource resolution](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-5.0.0_r1/core/jni/android_util_AssetManager.cpp).

Explicit null src/srcCompat clears ImageView bytes and Drawable identity without
looking up resource zero. Programmatic replacement/clearing and GC pass compiled
checks; ordinary Resources.getDrawable(0) still rejects an invalid resource.
TypedValue.getFloat returns the data field's float bits, including signed zero,
subnormals, infinity and NaN payloads; actual resolved theme floats and GC pass.
TypedArray.getValue now fills supported output fields, resolves scalar/string
aliases to their last resource ID and retains complex references. Missing/null
values leave the output unchanged. XML types, theme snapshots, managed strings,
invalid indices/outputs, resource cycles and recovery pass a compiled check.
These semantics were checked against [API-21 TypedValue](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-5.0.0_r1/core/java/android/util/TypedValue.java)
and [TypedArray](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-5.0.0_r1/core/java/android/content/res/TypedArray.java).
One virtual resource pool/default configuration supplies cookie/density metadata;
Android asset-cookie/qualifier provenance and recycled-array faults remain incomplete.

Recognized API-21 Material/Material.Light and NoActionBar themes now supply
flat default primary/secondary/hint text colors, disabled alpha and 56dp action-bar
size through the shared explicit/implicit parent resolver. Application styles
override those values. Numeric TypedArray getters resolve theme aliases against
their retained snapshot. Compiled checks cover light/dark defaults, overrides,
float/dimension aliases, layout flags, subpixel sizes, snapshots and GC.
getLayoutDimension decodes resolved sizes instead of using encoded IDs; resource
and typed-array pixel sizes share API-21 rounding. Theme explicit-style calls
retain the same snapshot rules. Values were checked against the
[API-21 Material theme definitions](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-5.0.0_r1/core/res/res/values/themes_material.xml),
[color defaults](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-5.0.0_r1/core/res/res/values/colors_material.xml)
and [dimensions](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-5.0.0_r1/core/res/res/values/dimens_material.xml).
Full framework selector/theme resources remain incomplete. The unchanged SwpieView
APK's button styling exercises these inherited values when opening its image viewer.
Its headless decoded image occupies 420 by 664 points below the 56-point action bar;
the public replay also checks viewer bounds, Back and gestures.

The unmodified Notepad APK now completes its actual AppCompat dialog layout
inflation. Its original focused EditFolderViewHolder.clickLeftButton calls at
PC 0x0057; the bundled dialog onCreate advances from PC 0x0012 to PC 0x019d.
The next exact failure is
`unsupported method Landroid/support/v4/widget/NestedScrollView;->setClipToPadding(Z)V`.
The permanent public replay checks that stack location and keeps confirmed folder
deletion false. Its disposable attempt preserves the same saved folder ID/name,
both exact note IDs/titles/bodies and the seed. The original APK SHA-256 remains
`2c35d3dc1d41d2c761b52785c591973886fb671a2cc2e7ab047ede89599db47f`.
Public presentation, confirmation, Cancel and physical folder interaction remain
unverified; no APK patch or replacement confirmation is used.

The complete local gate passes 142 Rust tests, warning-free Clippy, optimized
workspace/document-replay builds, 4,096 seeded parser mutations and all four
native font/focus/dialog/foreground component checks. GitHub Actions stay disabled.
The full public replay certifies one unchanged optimized CLI, including calculator,
image/grid/result, SwpieView and Notepad persistence/delete/Undo/drawer/folder
workflows plus the dialog diagnostic. That CLI and the unsigned development
bundle are byte-identical: SHA-256 `523d33070d626864073c5372b2c5ce83d29808647ac8c297b0591578f1797d74`.
The 50% checkpoint remains active; no new release or physical public-dialog
interaction is claimed.

## Current-source clipping and scroll-query dialog boundary

Date: 2026-10-02. Source increments: 9396b64 (native clipping), 1e83566
(virtual scroll queries), 0c77360 (foreground/drawing padding separation) and
b412e65 (permanent public replay). These increments are beyond v0.1.0.

ViewGroup clipping flags and XML booleans feed separate drawing and input
rectangles. Native layer masks clip descendants and foregrounds without cropping
control frames. Input follows ancestor bounds independently of padding/drawing
flags. Foreground padding still reserves layout space; only the View's own
padding controls the drawing clip. Compiled DEX checks cover nested flags, XML,
zero padding, actual padded Drawable callbacks, GC and invalid receivers. Native
checks sample clipped painted pixels, preserve editor selection, verify button
orientation/input, and exercise the same path on real Dialog panels. EdgeEffect,
scroll-coordinate clipping, arbitrary transforms and custom Canvas overflow
remain outside this profile. [Methods and references](framework.md#viewgroup-drawing-clips).

View.canScrollVertically/canScrollHorizontally call actual virtual offset, range
and extent methods in order, including APK overrides. Default metrics use layout
size and retained scroll offsets. Compiled checks cover both axes, overscroll,
end/empty/negative ranges, zero direction, Java integer wrapping, callback order,
GC, faults and recovery. Host scrolling and Android scrollbar rendering remain
outside this query profile.

The unmodified Notepad APK passes its original focused saved-folder delete
listener, dialog creation/start/guest attachment and both queued scroll-indicator
queries. Its next exact failure occurs during title measurement:
`unsupported method Landroid/text/Layout;->getEllipsisCount(I)I`.
The stack retains DialogTitle.onMeasure at PC 0x0012 under
ContentFrameLayout.onMeasure at PC 0x007d. The permanent replay requires the
original listener, clipping/start/attachment traces and both query directions,
then checks that the same folder ID/name, both exact note IDs/titles/bodies and
the seed remain unchanged. APK SHA-256 remains
`2c35d3dc1d41d2c761b52785c591973886fb671a2cc2e7ab047ede89599db47f`.
Guest attachment does not establish public presentation: measurement still fails,
and public confirmation, Cancel, confirmed folder deletion and physical folder
input remain unverified. No APK patch or replacement confirmation is used.

The complete local gate passes 144 Rust tests, warning-free Clippy, optimized
workspace/document-replay builds, 4,096 seeded parser mutations and five native
font/focus/dialog/foreground/clipping component checks. The full public replay
passes calculator, image/grid/result, SwpieView and Notepad workflows, including
the new diagnostic, against one unchanged optimized CLI. Its unsigned development
bundle is byte-identical: SHA-256
`613eebd7e8b22b76ed271841133937f73d4336995b06665fcc02969b8fb4c849`.
GitHub Actions remain disabled. The 50% checkpoint remains active; no new release
or physical public-dialog interaction is claimed.

## Current-source public folder deletion and text ellipses

Date: 2026-10-02. Source increments: 681d639 (text ellipses), 0d69186
(native accessibility check) and 23724f1 (permanent public confirmation replay).
These increments are beyond v0.1.0.

The shared scalar-width TextView layout now implements get/setEllipsize and
Layout.getEllipsisStart/getEllipsisCount. XML line and ellipsis settings feed
that state. Layout text retains UTF-16 length through ellipsis/filler replacement;
source text and old snapshots remain intact. Native controls receive shortened
visible text while keeping full accessibility labels. Compiled DEX checks cover
START/MIDDLE/END, fitting/empty/zero-width text, surrogate pairs, line limits,
XML, editable/read-only policy changes, invalid indices/enum objects and GC.
The expanded AppKit font check verifies shortened button/label values, full
accessibility labels and replacement. Shaping, styled metrics, Android font
parity, single-line transformations and marquee animation remain outside the
profile. [Methods and references](framework.md#guest-layout-callbacks-and-xml-metadata).

The unmodified Notepad APK now completes DialogTitle measurement and displays
its original headless confirmation: “Delete folder?”, the named-folder warning,
Cancel and Delete Folder. Both buttons retain actual guest listeners. The
permanent replay requires the original saved-row listener, clipping/start/guest
attachment, both scroll-query directions and the actual Layout ellipsis getter.
It then exercises Cancel, restart and Back, preserving the exact folder ID/name.
Confirmation dismisses the original Dialog and removes the saved Folder row.
A fresh process reopens Edit Folders without that row; Back restores both exact
note IDs/titles/bodies. Every phase checks SQLite and the untouched seed.
This proves the tested saved-folder workflow with two existing notes; it does
not establish physical public-dialog input or broader folder-assignment behavior.
APK SHA-256 remains
`2c35d3dc1d41d2c761b52785c591973886fb671a2cc2e7ab047ede89599db47f`.
No APK patch or substitute confirmation is used.

The local gate passes 145 Rust tests, warning-free Clippy, optimized
workspace/document-replay builds, 4,096 seeded parser mutations and five native
font/focus/dialog/foreground/clipping checks. The expanded native font check
also passes separately. The complete public replay passes calculator,
image/grid/result, SwpieView and Notepad workflows, including the original
confirmation/Cancel/deletion sequence, against one unchanged optimized CLI.
The unsigned development bundle is byte-identical: SHA-256
`76f48bbc9a356cd0d62ccdd6ee6befc491bcefa7485608eab4956a219e459beb`.
GitHub Actions remain disabled. The 50% checkpoint remains active; no new release
or physical public-dialog interaction is claimed.

## Current source: bounded private file input and backup diagnosis

Date: 2026-10-02. Runtime increments 2cf3047 (virtual external directory),
488375c (private input) and ee02770 (shared input channel); version 0.2.0 is
prepared in 8606e6e.

Environment.getExternalStorageDirectory now returns a real virtual guest File
whose directory maps to the current package's existing directory capability.
This profile grants no shared host volume or cross-package data. Private and
virtual external FileInputStream constructors read regular, no-follow files
with hard-link rejection and a 64 MiB ceiling. Construction snapshots bytes;
replacement leaves an existing stream unchanged. Missing/outside/oversized and
linked paths fail without initializing the receiver.

getChannel retains one managed input channel per stream. Shared size, position,
EOF and close state pass compiled DEX checks, including negative positions,
seeking beyond EOF, a 1 << 40 position, inherited interface identity, catchable
ClosedChannelException/IOException and GC of the source/channel cycle. The
existing stream path now copies only the requested read slice rather than the
entire snapshot on every read, skip or available query. Output, transfer,
ByteBuffer I/O, mapping, locking and live descriptors remain unsupported.

The original SHA-256-pinned Notepad 1.0.0 APK's Backup data listener now opens its
actual private AppDatabase.db input and obtains that channel. Its next explicit
failure is unsupported class `Ljava/io/FileOutputStream;`, in
`Lir/cafebazaar/notepad/activities/home/i;->a(Landroid/view/MenuItem;)Z` at DEX
PC 0x0091. Missing-file Restore data stops at
`Landroid/widget/Toast;->makeText(Landroid/content/Context;Ljava/lang/CharSequence;I)Landroid/widget/Toast;`
in the same listener at PC 0x00eb. This is diagnosis, not successful backup/restore.

The permanent public replay takes separate copies of the two-note/named-folder
seed. Both diagnostic failures preserve every column of the Note and Folder
tables in the copy and seed. The external directory contains no backup file.
The catalog/report retain explicit false backup and restore workflow flags.

`sh tools/ci.sh` completed with 147 Rust tests, formatting/checks, warning-free
Clippy, optimized builds, 4,096 seeded parser mutations and five native component
checks. `python3 tools/compatibility.py` completed the full public suite, including
the new diagnostic, original folder deletion, exact note persistence and malformed
XML preservation. The certificate records one unchanged optimized CLI SHA-256:
`8ac84853d73959bc1ff9e901639a5363651dd11518fb0396542e5434fb7f6f61`.
The rebuilt unsigned development bundle has the same executable. GitHub Actions
remain disabled; the 50% goal stays active. No new physical public-app input is
claimed for this increment.

## v0.2.0: clean extracted release validation

Date: 2026-10-02. The macOS ARM64 package requires the successful public-replay
certificate for the exact executable before it can be assembled. A clean
temporary extraction verifies the executable hash, Counter's real Increment
callback, same-APK detail navigation and Back, UTF-8 preference save/restart,
and the unchanged public Notepad APK's note save/restart. The Notepad artifact is
checked against its pinned SHA-256 and copied into the consumer test directory
after extraction; it is never included in the tarball. All these checks passed.

The tarball contains the CLI, license, five authored fixtures, three upstream
APK fetch/check helpers, README.txt and RELEASE.json. That record identifies
the executable SHA-256 and source revision; SHA256SUMS verifies the entire
archive. The unsigned development app bundle matches the same optimized CLI.
Earlier native public-app observations are retained separately; this release
gate does not claim new physical public-app input. Backup/restore remain
unfinished and the 50% goal remains active. GitHub Actions stay disabled.

[v0.2.0 scope and download](releases/0.2.0.md).

## Current source: bounded output and Notepad backup/restore

Date: 2026-10-02. Runtime checkpoint: `64017cc`.

`FileOutputStream` now accepts String/File paths in the current package's private
or virtual external directory. Writes are staged up to 64 MiB and committed
through the existing atomic storage path on flush/close. Append mode, byte-array
offsets, `getChannel`, shared stream/channel position, close propagation,
`transferFrom`, `transferTo` and unsafe host-path rejection run in a compiled
DEX contract. An abandoned unclosed stream leaves the earlier file unchanged.
File listing can run the APK's actual `FileFilter.accept` over the confined
storage entries.

The unchanged, SHA-256-pinned Notepad 1.0.0 APK now completes its Backup data
callback. Its `external/notepad_backup.nbu` equals the private SQLite database
byte-for-byte: 24,576 bytes, SHA-256
`fcd71e0e647dede86efc58bf768bfbb3e7fefab7c5621a744ec6ad4ab2f19c09`. On a
separate copy, the test alters one Note row, selects the backup through the real
file-picker row and presses the APK's Restore confirmation. The original bytes,
SQLite integrity, every Note row and the Folder row are recovered; the next
process renders the saved notes and folder. The test seed stays untouched.

After committing the restored database, this APK calls `System.exit(0)`. That
method is still unsupported, so DROIDLESS reports a guest error after the data
recovery has succeeded. This evidence is a bounded workflow check, not complete
process-shutdown or Android compatibility. Custom Bundle Serializable objects
are same-runtime references, not Java serialization or cross-process persistence.
