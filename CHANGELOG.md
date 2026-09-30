# Changelog

## Unreleased

- Deferred guest Thread.start execution on a serial host worker executor, retaining
  shared-heap DEX frames across LinkedBlockingQueue take/put and reentrant monitor
  waits. Stable identities, start-once faults, interrupt delivery, GC roots, bounded
  slices and teardown are checked. Main Handler result delivery passes headless
  replay; direct worker UI access is rejected. The pure Java WorkerContract also
  passes on desktop Java. Main waits, native bridge/initializer suspension, worker
  Looper delivery, priority, sleep/join and parallel CPU execution remain unsupported.

- Iterative managed DEX calls with return continuations and cross-frame exception
  unwinding. Compiled checks pause/resume nested calls and collect after each step;
  reference/wide results, catch/finally, diagnostics and stack limits remain intact.
  The normal FrameContract also passes on desktop Java. Native bridges/class
  initialization remain synchronous; Thread.start/waits are still unsupported.
- Immediate LinkedBlockingQueue FIFO operations with declared capacity, duplicates,
  null rejection, guest equality, inherited override dispatch and GC retention.
  The same immediate contract passes in compiled DEX and desktop Java. Waiting
  and workers remain unsupported. Local CI passes 33 Rust tests, 4,096 mutations
  and 17 original calculator scenarios.
- Unmodified Notepad resolves queue construction and now stops at Thread.start in
  DBFlow startup, before Activity/UI creation. No initializer is skipped.

- Main Handler/Looper/Message queue with deferred/delayed APK callbacks, identity
  cancellation, virtual dispatch, GC roots and bounded clock/queue execution.
  Native authored timer, cancellation and delayed finish verified; deterministic
  `--advance-ms` replay and unstarted Thread metadata/manual run support.
  Thread.start and blocking queues remain unsupported. Local CI passes 32 Rust
  tests, 4,096 parser mutations and 17 original calculator scenarios.
- At the scheduling checkpoint unmodified Notepad passed Thread(String) and stopped at
  LinkedBlockingQueue in DBFlow startup, before Activity/UI creation.

- Bounded ArrayList with ordered duplicates/nulls, indexed operations, guest
  equality and Set/List iterators; basic inherited LinkedHashMap operations.
  The compiled list contract also passes on desktop Java. That checkpoint passed
  28 Rust tests, 4,096 parser mutations and 17 original calculator scenarios.
- At the collections checkpoint, unmodified Notepad passed DBFlow's collection
  constructors and stopped at Thread(String), before Activity/UI creation.

- Replace the public README/site showcase with unmodified Simple Calculator 1.0,
  a real white/charcoal native capture and seven checked headless scenarios.
  Site/SVG accents use blue instead of peach; the previous pink capture is removed.
- Boxed Double valueOf/unboxing/toString/isNaN and Long.toString(J), plus bounded
  `--size WIDTHxHEIGHT` native/headless viewports. Local CI passes 28 Rust tests.
- APK-local Class lookup/no-argument construction, guest access/initialization
  faults and inherited field resolution. Compiled conformance plus a desktop Java
  differential run. Notepad stopped at ArrayList at that checkpoint; no notes UI claim.

- Bounded HashSet/HashMap with guest equals, Set iterators, live unmodifiable Set
  views, GC retention and explicit unsupported methods. Compiled conformance
  and capacity/error regressions brought that increment to 24 Rust tests.
- Canonical Class literal identity and basic package metadata. Unmodified Notepad
  passed collection setup and stopped at Class.forName at that checkpoint.

- Isolated typed SharedPreferences with staged editors, atomic persistent writes,
  package/case/link checks and explicit storage ceilings. `--data-dir`,
  `--ephemeral` and headless `--input` controls.
- Authored preferences APK verifies native UTF-8 paste/save/restart/clear and
  five persistence/error/isolation regressions. Standard AppKit Edit menu fixes
  native paste; apply remains synchronous pending scheduling support.
- At the storage checkpoint, Notepad 1.0.0 reached DBFlow and stopped at HashSet;
  the collections increment above records its current diagnosed startup failure.

- Playful lime/ink identity, custom SVG robot/banner/runtime diagram, refreshed
  README and portable website with self-hosted OFL fonts and accessible motion.
- Next working checkpoint raised to 50%; milestone labels remain separate from
  measured Android API coverage.

- Explicit same-APK Activity Intents, typed Bundle extras, copied launch data,
  preserved back stack, finish/isFinishing and virtual Back callbacks.
- Native screen/title changes and Escape-to-Back; `--back` headless replay and
  clean termination when the root Activity finishes.
- Authored Intents conformance APK and native navigation verification. v0.1.0
  release artifacts retain their original scope. CI remains local only.

## 0.1.0 — 2026-09-30 — interactive APK preview

- Unmodified KasCalc 1.0 APK runs DEX math/click callbacks in a native macOS window;
  real screenshot and ten deterministic headless scenarios.
- Register VM, managed heap/mark-sweep, objects/fields/calls, numeric/wide/array
  opcodes and explicit exceptions, with compiled conformance tests.
- Activity lifecycle, binary layouts/resources, native widgets/input bridge,
  exact unsupported method/opcode diagnostics and local-only CI.
- Catchable implicit Java faults, inherited interfaces and reference array
  covariance, verified by 17 compiled fault paths and catch-all/finally.
- Persistent failed-class initialization, Java error wrapping and GC-rooted causes.
- Native Counter text/key delivery and clean window close; unmodified KasCalc
  rechecked through native 7 + 5 and lifecycle close with process exit status 0.

## 0.0.1 — inspection foundation

- APK/binary manifest/XML/DEX/resources parsing and inspection CLI.
- Bounds/digests/malformed-input tests and unmodified third-party sample fixture.
