# Changelog

## Unreleased

- Packaged PNG/JPEG/WebP now flow through BitmapFactory resource, stream and
  byte-array paths into AppKit ImageViews. An authored fixture verifies bounds,
  sampling, XML src, drawable assignment and four native images. Independent
  image-app workflows, bitmap pixel operations and Canvas remain open. Local
  CI passes 69 Rust tests and 4,096 seeded parser mutations; GitHub Actions stay
  disabled.
- The unmodified public Notepad APK now saves two titles and renders both in
  Notes after Back and a fresh DROIDLESS process. Native AppKit keyboard entry
  also reaches the APK and persists through its save path. Targeted RecyclerView
  support adds generic View scroll/child operations and `Long.rotateRight`; local
  compatibility checks pass while broad list rendering remains limited. GitHub
  Actions stay disabled.
- At the previous checkpoint, the unmodified public Notepad APK opened its
  editor and saved the edited title to SQLite; the row survived a fresh
  DROIDLESS process, but the reopened Notes list still showed its empty state.
  Added Android text/SAX support,
  reflection interface lookup, Java array binary search and SQLite updates for
  this save path. Local CI passes 66 Rust tests, Clippy, release build and 4,096
  seeded parser mutations.
- Fixed read-only Build.VERSION.SDK_INT = 21, independent of APK/host metadata.
  Compiled checks cover stable reads, inherited aliases and native final-field
  faults. This branch profile does not imply full API-21 compatibility.
- Application lifecycle-observer registration/removal and GC-rooted snapshot
  delivery from six Activity super methods; canonical getApplication identity.
  GC during observer callbacks exposed and fixed roots for active navigation
  actions and registered Activities. Compiled checks cover reentrant registration,
  navigation/Back/close, retention/release and callback fault cleanup. Native
  navigation/Back/close executes 33 observer calls and exits with status 0.
  Saved-state/pre/post callbacks and missing-super enforcement remain unsupported.
- At an earlier checkpoint, unmodified Notepad passed SDK checks and observer
  registration, then reached FileInputStream while Stetho read /proc/self/cmdline.
  That checkpoint had no Activity/UI workflow; it passed 41 Rust tests and
  4,096 mutations.

- Bounded CopyOnWriteArrayList operations and snapshot iterators: old values
  survive live mutations, GC and serial guest worker updates; iterator removal
  raises UnsupportedOperationException. Read-only views preserve snapshots. The
  normal SnapshotContract also passes on desktop Java. Reentrant remove equality,
  copy constructors, bulk APIs and ListIterator/subList remain unsupported.
- At the snapshot-list checkpoint, unmodified Notepad passed construction and stopped at
  Build.VERSION.SDK_INT in Application.onCreate, before Activity/UI creation.
  Local CI passes 40 Rust tests, 4,096 mutations and 17 calculator scenarios.

- Canonical primitive Class metadata from all nine wrapper TYPE fields; wrapper
  lookup, constructor faults and rejected native final-field writes. The compiled
  PrimitiveContract also passes on desktop Java.
- Native HashMap/LinkedHashMap putAll with guest equality and GC-rooted snapshots;
  live unmodifiableList views reuse the read-only collection bridge. Read-only
  iterators now delegate without disabling a mutable alias. Compiled contracts
  also pass on desktop Java. Other bulk APIs, custom Map copying and ListIterator/
  subList remain unsupported.

- At the metadata/map-copy checkpoint, unmodified Notepad passed primitive
  metadata, native map copying and unmodifiableList setup, then stopped at
  CopyOnWriteArrayList before Activity/UI creation. No initializer is skipped and
  the original APK remains unchanged.

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
  initialization remain synchronous; Thread.start/waits were unsupported at that
  checkpoint.
- Immediate LinkedBlockingQueue FIFO operations with declared capacity, duplicates,
  null rejection, guest equality, inherited override dispatch and GC retention.
  The same immediate contract passes in compiled DEX and desktop Java. Waiting
  and workers were unsupported at that checkpoint. Local CI passed 33 Rust tests,
  4,096 mutations and 17 original calculator scenarios.
- At the immediate-queue checkpoint, unmodified Notepad resolved construction
  and stopped at Thread.start in DBFlow startup, before Activity/UI creation. No initializer is skipped.

- Main Handler/Looper/Message queue with deferred/delayed APK callbacks, identity
  cancellation, virtual dispatch, GC roots and bounded clock/queue execution.
  Native authored timer, cancellation and delayed finish verified; deterministic
  `--advance-ms` replay and unstarted Thread metadata/manual run support.
  Thread.start and blocking queues were unsupported at that checkpoint. Local CI
  passed 32 Rust tests, 4,096 parser mutations and 17 original calculator scenarios.
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
