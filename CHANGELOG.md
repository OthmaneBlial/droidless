# Changelog

## Unreleased

- ContextThemeWrapper now copies the base theme into an independent cached theme,
  invokes actual guest theme callbacks and clones the base inflater with its
  factories and wrapper context. Supported ContextWrapper calls delegate to the
  actual base. New themes start empty; Theme.setTo retains destination ownership.
  Compiled checks cover isolation, immutable arrays, default IDs, GC, faults,
  invalid constructors and bounded recursive wrappers. Dialog windows and folder
  deletion remain unsupported or unverified.

- Native editor first-responder transitions now request actual guest View focus.
  AppKit retains selection; rejected focus cancels editing and callback faults
  stop the host. Drawing suppresses focus reentry. The CLI adds --focus-at INDEX
  through the same dispatcher. Compiled callback/GC/fault checks and native
  first-responder/key-view/selection checks pass; physical folder input remains
  unverified.

- Paint/TextPaint now retain text size and expose ascent/descent measured from
  the same AppKit fonts used by native controls. Compiled checks cover all
  supported families/styles, size scaling, zero/negative sizes and GC. The
  unchanged Notepad APK completes folder rename; restart and Back preserve the
  folder ID/new name and both exact notes. Android font parity, folder deletion
  and physical native folder input remain unverified.

- Optimized public replay also checks saved-folder focus, pending input and
  restart discard while preserving exact notes and the saved Folder ID/name.

- Paint/TextPaint shadow configuration now retains radius, offsets and ARGB
  color, exposes layer state and clears through the virtual setter. Canvas
  shadow rasterization and native shadow visual parity remain unsupported.

- Color channel extraction and RGB/ARGB packing now follow API-21 Java int bits,
  including signed colors. Saved-folder focus reaches the original collapsed
  label color calculation. Compiled bridge checks cover channel bytes and packing.

- ViewGroup convenience overloads now dispatch the APK's real indexed binding
  callbacks during inflation. Cached Typeface state reaches native AppKit fonts.
  TextWatcher delivery uses real guest callbacks, UTF-16 deltas, retained buffers
  and fault cleanup; standalone scalar ValueAnimator callbacks use the existing
  runtime clock. The unchanged Notepad folder row now runs its actual
  TextInputLayout binding path. Asset fonts, general animation APIs and physical
  folder input remain unverified or unsupported.

- Typed colors now resolve through actual guest Context/Theme callbacks and copied
  theme snapshots. TextView appearance applies color and pixel size; unresolved
  theme values raise the catchable API-21 RuntimeException. Framework white, black
  and transparent colors resolve through APK aliases. Sized child attachment
  invokes virtual default factories and indexed addView with GC/fault cleanup.
  The unchanged Notepad APK creates a visible folder, reopens it after restart and
  returns to both exact notes through Back. Editing/deletion and native folder
  input remain unverified.
- TextPaint now shares native Paint inheritance, constructor flags/color state
  and canonical public fields. Child drawable-state aggregation invokes actual
  guest callbacks, retains arrays through GC, propagates faults and rejects
  cycles/deep trees. Recursive state calls use the small UI dispatcher. The
  unchanged Notepad saved-folder row now passes error-label text appearance.
- Descendant Rect conversion now follows managed layout/scroll state in both
  directions, with signed Java overflow, ancestry faults and bounded chains.
  Rect dimensions and resource backgrounds use real Context/Resources/View
  callbacks, managed identity, cache invalidation and GC/fault recovery. Untinted
  background queries are supported; tint application remains ahead.
- Throwable logging now retains actual DEX diagnostics, guest callbacks, GC and
  fault cleanup. The cached input_method service reports software show/hide as
  unavailable in the hardware-keyboard profile. Generic editor touch focus
  invokes the original Notepad focus/Done callbacks and persists one named folder.
  New-row layout now completes and both exact notes remain unchanged. Folder
  editing/deletion and native folder input remain unverified.

- Native Notepad now selects an existing row with the mouse, accepts keyboard
  title/body edits, saves through Back and reopens both exact fields after a
  fresh process. The original SQLite row ID is retained and native close exits
  cleanly. AppKit hit testing keeps editable controls on native focus/selection.

- Layout now calls real inherited APK onMeasure/onLayout callbacks before
  native drawing and root touch. XML retains AttributeSet/Context, uses virtual
  parent layout parameters and supports attached merge and ViewStub replacement.
  The closed Notepad drawer is offscreen; invisible ancestors exclude native
  descendants. Shared measurement/geometry, per-edge padding, resource color
  selectors and reference-only reflected constructors pass compiled contracts.
  Compound drawable/tint/checkmark painting and complete Android focus/styling
  remain outside this profile.

- The unchanged public Notepad APK now reopens an existing note, updates its
  title and multiline body, refreshes the list and restores both fields after
  restart without creating another row. Host text selection finds a label's
  nearest click owner; --input-at selects another editor field. Boxed extras
  retain shallow-copy identity and use ordinary Parcel value tags. Object-array
  sorting shares stable guest comparison callbacks with Collections.sort;
  bounded TextUtils search/replacement supports the body serialization path.
  Typed SharedPreferences retain their original storage representation.
  UNSPECIFIED measurement now preserves intrinsic card sizes; bounded
  VelocityTracker support shares the gesture estimator. That earlier host-replay
  checkpoint did not verify native existing-note selection/editing.

- Executor.execute no longer runs tasks inline or reports fabricated shutdown
  success. Single/fixed/cached pools reuse guest workers; Callable and Runnable
  submissions retain real Future values/causes, cancellation and deadline waits.
  Shutdown drains accepted work; shutdownNow returns actual queued tasks.
  Compiled contracts cover GC, interrupts, replacement, capacity and close;
  the authored native wait/deliver/cancel flow posts UI results to main.

- Java Timer/TimerTask scheduling now uses one stable guest worker per Timer,
  with long/Date deadlines, fixed-delay/fixed-rate tasks, catch-up, cancellation,
  purge, serial blocking, GC roots and failure cleanup. Compiled contracts cover
  main Handler UI delivery, capacity and shutdown; the portable validation/serial
  task contract also passes on desktop Java.
- FrameLayout XML/parameter gravity now places SwpieView controls below its
  toolbar. Root touch replay diagnoses the original APK's slideshow: DOWN starts
  its timer, UP cancels it, and a held DOWN reaches a worker UI call that is
  explicitly rejected. A usable public slideshow remains unproven.

- The unmodified public Notepad APK now renders a saved title in the reopened
  Notes list after Back and after a fresh DROIDLESS process. Local verification
  also keeps the native-window boundary explicit; GitHub Actions stay disabled.
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
