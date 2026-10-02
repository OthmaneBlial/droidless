# Compatibility-driven next milestones

The first third-party interactive calculator milestone is proven on macOS ARM64.
The following are objectives, not completed capability claims.

1. Continue VM verification: full field/method reference validation and instruction
   boundaries. Gate: compiled conformance/malformed-input regressions.
2. Improve input/resources: full focus/IME/paste, qualifier/style resolution and
   weighted measurement remain open. Packaged PNG/JPEG/WebP decoding, native
   ImageView rendering and a bounded `Resources.getXml` pull cursor pass authored
   fixtures. SwpieView now passes AppCompat's bundled vector resource check in
   guest DEX, attaches its platform lifecycle fragment and constructs its toolbar,
   builds its GridView and BaseAdapter and opens a native folder chooser.
   Cancellation exits cleanly; selected-folder queries, guest sorting and read-only
   image streams now display three PNG/JPEG/WebP thumbnails in the unmodified APK's
   native grid. Selecting a thumbnail now runs actual guest Parcelable/CREATOR
   callbacks and opens the selected image in its full-screen Activity. Native
   JPEG/PNG/WebP viewer visits and Escape back to the grid are verified.
   Authored grids now verify native image clicks, disabled items, observer updates
   and auto-fit/stretch geometry; viewport recycling and scrolling remain open. Independent-app
   vector/animated drawables, pixel operations and Canvas remain open. Genuine
   single-pointer input now runs SwpieView's own swipe and confirmed tap callbacks
   in host replay, with first/last bounds and hide/show checks. Android differential
   input validation, multi-touch and full interception remain open.
3. Multi-screen/persistence: explicit Intents/Bundle/back stack are proven in an
   authored native fixture. Activity result snapshots, request codes, cancellation
   and deferred caller delivery now pass compiled checks; native folder selection
   returns an actual granted URI to an authored APK. Isolated SharedPreferences save/restart/clear are also
   proven in an authored native fixture. Basic HashSet/ArrayList/HashMap/LinkedHashMap
   and live read-only Set/List views pass compiled conformance. Native map bulk
   copying and primitive Class metadata now complete more DBFlow setup in the
   unmodified Notepad. Snapshot CopyOnWriteArrayList construction, the fixed API-21
   profile, lifecycle observers and its DBFlow startup now resolve. The Notes list
   and note editor render headlessly; tapping ＋ and entering text changes the
   original APK's editable View tree. Saving the title writes the APK's private
   Note table; the headless replay creates two notes and renders both titles
   after Back and after a fresh process. A native AppKit window also accepted
   keyboard text and saved the resulting note. Existing-note row selection and
   title/body editing now also retain the same SQLite ID and reopen both fields
   after restart in host replay. Native mouse selection and keyboard edits now
   also save back to the list and reopen both exact fields after a fresh process.
   Guest layout callbacks preserve closed-drawer geometry; native editor clicks
   retain AppKit focus and selection. Rich formatting/drawing, broader list
   behavior and visual fidelity remain open.
   Foreground options now run actual create/prepare/listener/selection callbacks
   through headless menu replay, with invalidation, stale input and GC checks.
   Notepad's original Delete removes the intended row. Its Snackbar now completes
   child binding through onFinishInflate and delivers its queued confirmation
   callback and completes detached-child removal. Snackbar child measurement now
   creates its actual text Layout and reads its line count. Relative-padding state
   now resolves and Snackbar starts its original translation/alpha callbacks.
   Clock-driven View property frames, cancellation and callback cleanup pass a
   compiled contract. Headless Delete returns to Notes, preserves the survivor's
   ID/title/body after restart and reopens its exact fields. AppKit now bridges
   foreground options through the existing guest lifecycle/selection checks; a
   standalone native component contract passes. Physical menu input and
   native timed feedback/dismissal remain ahead. Headless replay shows the original
   deletion message and UNDO label at 250 ms, then removes the Snackbar after
   another 3000+250 ms with the exact surviving row retained. Headless Undo now
   restores title/body with a fresh auto-increment ID, preserves the survivor and
   reopens the restored fields after restart. The old timeout causes no further
   change. Physical Delete/Undo input remains ahead.
   Headless navigation taps now reveal the APK's drawer animation frame at 100 ms,
   retaining both exact note rows. Shared rendering runs guest
   computeScroll callbacks and attached animation redraw requests reach host
   polling. Generic focus ownership and callbacks now complete the 1000 ms drawer
   settlement callback. Back closes it while retaining Notes and both exact rows.
   Native drawer input remains ahead. Create or edit folders now opens Edit Folders
   and binds its original editor/button listener. Shared inflater callbacks run the
   APK's AppCompat Factory2 and construct its actual AppCompatImageButton; no APK
   patch or class alias is used. Compiled checks cover service routing, cloned and
   merged factories, attributes, ViewStubs, GC and callback faults. Starting folder
   creation now passes descendant coordinates, signed Rect dimensions and resource
   backgrounds. Generic touch focus opens the original editor/Done action and
   its callback persists one named folder. TextPaint and bounded child drawable
   states now pass. Themed text appearance preserves the APK's error-label fallback;
   sized child attachment invokes its virtual factory and indexed addView. Headless
   creation displays the saved folder, reopens it after restart and returns to both
   exact notes through Back. Real indexed binding now places the saved editor
   within its parent and accepts headless focus/pending input; restart discards
   an unconfirmed name. Typeface state reaches native fonts, TextWatcher callbacks
   execute real DEX, and scalar animation/color/shadow state passes checks. Rename
   confirmation now completes with actual host-font ascent/descent. The same
   folder ID/new name survive restart and Back while both notes stay exact.
   Software IME and native folder input remain ahead; Android
   font parity is not established.
   Native editor first-responder gains now invoke real guest focus callbacks;
   component checks cover key-view traversal, selection and refusal/failure.
   The same dispatcher creates and renames folders in optimized host replay.
   Physical folder input and complete bidirectional focus remain unverified.
   Generic modal Dialog surfaces now use separate native panels with guest lifecycle,
   input, nesting, cancellation and GC checks. The public folder-delete listener
   completes its bundled AppCompat inflation and displays its original confirmation
   in headless replay. Cancel preserves the folder; confirmed deletion, restart
   and Back preserve both exact notes. Text ellipsis offsets/snapshots and native
   display projection have compiled checks. Physical public-dialog input remains ahead.
   Private FileInputStream snapshots, staged package-confined FileOutputStream
   writes and shared bounded input/output channel transfers pass compiled checks.
   The unchanged Notepad APK now writes a byte-exact SQLite backup, restores a
   tampered copy and renders every Note/Folder row after a fresh launch. Guest
   System.exit(status) unwinds DEX frames without entering Java catch handlers;
   the CLI returns that status. The restore replay exits with status 0. The v0.3.0
   archive predates this follow-up and still reports the earlier exit boundary.
   General file APIs and native folder/dialog input remain future work.
   The neutral public calculator continues to prove boxed Double execution
   through native clicks.
4. Async Java: main Handler/Looper/Message scheduling and native authored timer
   callbacks are proven. Immediate FIFO queue operations also pass compiled and
   desktop Java conformance. Managed DEX call continuations now pause/resume with
   GC and exception checks. Bounded deferred workers now execute on a serial shared-heap
   host executor, with queue/monitor waits and main Handler result checks.
   Java timers now reuse one guest worker per Timer: deadlines, catch-up,
   serial tasks, cancellation/purge, failure cleanup and main Handler posting
   pass compiled contracts. Public SwpieView's ordinary touch starts/cancels its
   timer; holding it exposes its worker UI call. Usable slideshow is not proven.
   Single/fixed/cached pools now queue work on reusable guest workers; Future
   values/causes, cancellation, deadline waits, shutdown and main Handler posting
   pass compiled checks. The authored native wait/deliver/cancel flow is verified.
   Prepared worker Loopers now deliver messages through managed DEX callbacks,
   suspend on supported worker waits, and support quit/quitSafely. Routing, order,
   cancellation, GC, exception recovery and bounded dispatch pass authored checks.
   Native bridges/class initialization remain synchronous; main waits, nested
   message pumps and priorities remain ahead. Worker sleep and timed/indefinite
   joins now preserve frames, monitors and interrupts; compiled checks and a
   separate desktop Java contract pass. Main blocking waits remain unsupported.
   Notepad passes deferred Thread.start;
   independently completed transaction execution remains unproven.
   Gate: a real APK completes its queued transactions and posts UI results.
5. Isolated file/process APIs, broader lists/SQLite/network and independent image-app workflows: follow first failures in substantial third-party
   apps, with host capabilities explicit and tested. Current SwpieView startup
   loads a selected folder into a native thumbnail grid, opens static images and
   returns through Back; host replay also verifies static-image swipes and taps.
   GIF animation, slideshow and broader provider access remain open.
6. Linux native backend: common VM/View model, real Linux builds/render/input.
7. Kotlin/AndroidX, then fragment Views/Compose/JNI/foreign libraries: major subsequent
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
