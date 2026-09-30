# Compatibility-driven next milestones

The first third-party interactive calculator milestone is proven on macOS ARM64.
The following are objectives, not completed capability claims.

1. Continue VM verification: full field/method reference validation and instruction
   boundaries. Gate: compiled conformance/malformed-input regressions.
2. Improve input/resources: full focus/IME/paste, qualifier/
   style resolution, images/drawables and weighted measurement. Gate: native
   interaction and comparisons with expected Android behavior.
3. Multi-screen/persistence: explicit Intents/Bundle/back stack are proven in an
   authored native fixture. Isolated SharedPreferences save/restart/clear are also
   proven in an authored native fixture. Basic HashSet/ArrayList/HashMap/LinkedHashMap
   and live read-only Set/List views pass compiled conformance. Native map bulk
   copying and primitive Class metadata now complete more DBFlow setup in the
   unmodified Notepad. Snapshot CopyOnWriteArrayList construction now resolves;
   startup stops at Build.VERSION.SDK_INT in Application.onCreate before Activity
   creation. APK-local lookup/reflective
   construction and inherited fields also resolve. The neutral public calculator proves boxed Double
   execution through native clicks. Follow that failure, then the actual storage/
   UI needs of a real notes/todo APK with restart/persistence tests.
4. Async Java: main Handler/Looper/Message scheduling and native authored timer
   callbacks are proven. Immediate FIFO queue operations also pass compiled and
   desktop Java conformance. Managed DEX call continuations now pause/resume with
   GC and exception checks. Bounded deferred workers now execute on a serial shared-heap
   host executor, with queue/monitor waits and main Handler result checks. Native
   bridges/class initialization remain synchronous; main waits, worker delivery,
   priorities and join/sleep remain ahead. Notepad passes deferred Thread.start;
   independently completed transaction execution remains unproven.
   Gate: a real APK completes its queued transactions and posts UI results.
5. Virtual system metadata, lists/images/SQLite/network: follow first failures in substantial third-party
   apps, with host capabilities explicit and tested.
6. Linux native backend: common VM/View model, real Linux builds/render/input.
7. Kotlin/AndroidX, then fragments/Compose/JNI/foreign libraries: major subsequent
   compatibility projects. No emulator fallback.
8. Profile before JIT/AOT, Canvas/GLES, audio or games; interpreter stays the
   correctness reference.

Every stable increment runs local CI, updates evidence/docs, commits and pushes
main. GitHub Actions stay disabled. Releases/demos name exact artifacts and scope.

## The next checkpoint: 50%

The user has raised the working target beyond the demonstrated 20% native
calculator milestone. These percentages are milestone labels, not Android API
coverage measurements. The 50% checkpoint remains future work.

Continue toward useful unmodified everyday apps: multiple screens, isolated
persistent storage, lists, images and scheduled callbacks. Validate generic APIs
with authored conformance fixtures, then prove actual workflows in independently
built APKs. Keep local CI, native interaction evidence, restart checks and honest
compatibility limits attached to each completed increment. Do not stop at the
calculator or substitute authored fixtures for third-party app evidence.
