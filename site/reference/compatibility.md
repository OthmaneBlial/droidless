# Compatibility and exact limits

## Implemented opcode groups

| Opcode range | Implemented group |
|---|---|
| 00–11 | NOP, move variants/results/exceptions, returns |
| 12–1c | Numeric/wide/string/class constants |
| 1d–27 | Guest-owned reentrant monitors, casts/type tests, arrays/instances, explicit throw |
| 28–3d | Goto, packed/sparse switches, numeric comparisons and conditions |
| 44–6d | Array and instance/static field reads/writes |
| 6e–72, 74–78 | Virtual/super/direct/static/interface calls and range forms |
| 7b–8f | Negation/complement and numeric conversions |
| 90–cf | Int/long/float/double arithmetic and two-address forms |
| d0–e2 | Integer literal operations |

Decoder coverage is not proof of every valid/malformed edge case. Tests cover
arithmetic boundaries and compiled loops, fields, class initialization, wide
values, arrays, dispatch and explicit/implicit throw/catch/finally. Complete verifier/debug/
annotation coverage remains pending. Reserved/newer opcodes fail with method/PC.

## Framework families reached by real execution/tests

Activity constructors/lifecycle/content/title/findViewById/window metrics;
Context/Resources strings/resources; View ID/visibility/enabled/background/
per-side padding/click/key listeners, scroll offsets and the default scroll-change
callback; ViewGroup add/detach/attach child operations; LinearLayout orientation;
TextView text/append/size/color/gravity; EditText text/null key listener/selection;
KeyEvent action/keycode. Long.rotateRight(JI)J is implemented for 64-bit values.
Platform Menu/MenuItem state and MenuInflater load packaged flat menu XML, with
category ordering, resource titles/icons, group flags, exclusive checks and GC
ownership verified by an authored compiled contract. Drawable.setTint delegates
to virtual setTintList and retains tint metadata; it does not paint tinted icons.
The unmodified Notepad constructs its delete menu through its own AppCompat code.
Headless `--menu-item TEXT` now prepares the foreground Activity's options and
dispatches visible/enabled items to guest listeners, then onOptionsItemSelected.
Compiled checks cover cache/invalidation, rejection, navigation, errors and stale
input. Selection never toggles a checked item automatically and a false callback
return does not undo guest effects. Native menu presentation/input remain open.
The public Notepad's Delete callback removes the intended SQLite row and preserves
the other row, then reaches its original Snackbar feedback. That path currently
fails at Math.max(float, float) in its queued Snackbar callback; a complete
delete/return/restart workflow is not yet verified. Generic `<view class="…">` layout inflation now invokes the
named APK View constructor, applies its XML attributes and invokes virtual
onFinishInflate after attaching its children. Compiled checks retain the subtree
through callback GC. View live-region mode bits are retained; Android accessibility
service announcements remain outside this profile. Class.toString formats
APK/native-profile classes, interfaces, primitives and arrays; unknown native
metadata remains explicitly unsupported. The primitive/class-description contract
also passes on desktop Java 17.
Submenus, shortcuts, XML onClick/action Views/providers and theme references remain
unsupported; each menu is bounded to 1,024 items. The ordering and XML defaults
follow the [API-21 MenuInflater reference](https://android.googlesource.com/platform/frameworks/base/+/android-5.0.0_r1/core/java/android/view/MenuInflater.java).
BitmapFactory decodes packaged PNG/JPEG/WebP through resource, stream and byte-array
paths; Bitmap bounds options, dimensions and ImageView resource/bitmap/drawable
assignment reach AppKit image views. `Resources.getXml` exposes a bounded binary
XML pull cursor and resource-ID-backed typed attributes, tested by an authored
APK. SwpieView passes its bundled vector configuration check and attaches its
platform ReportFragment, constructs its toolbar/GridView/BaseAdapter and opens a
native folder chooser. Selecting the owned image folder now runs its query,
guest sort and image-stream decode, displaying three thumbnails in AppKit and
closing cleanly. Selecting a thumbnail now executes Parcelable write/CREATOR
callbacks, opens its full-screen Activity and decodes the selected image through
ImageView.setImageURI. Native JPEG/PNG/WebP viewing and Escape back to thumbnails
are verified. Single-pointer host replay also runs the APK's own swipe navigation
and confirmed tap hide/show callbacks. GIF animation, slideshow and lifecycle
auto-hide remain unproven. MotionEvent/View/Activity dispatch and timed gesture
callbacks pass compiled guest checks; multi-touch, intercept behavior and Android
VelocityTracker parity remain unsupported. Bitmap pixel manipulation and Android
Canvas/vector drawing remain open.
FrameLayout child gravity now honors XML and explicit parameters with padding/
margins in the default LTR profile. SwpieView's bottom controls become reachable
through root touch dispatch. Its slideshow tap starts then cancels a real Timer;
a held DOWN runs the task on its worker and rejects its UI access.
Bounded Java Timer/TimerTask schedules, Date deadlines, fixed-delay/fixed-rate,
cancellation/purge and worker-to-main Handler posting pass compiled contracts.
[Timer scope and ceilings](threading.md#java-timers). Usable public slideshow,
Timer finalization and independent JVM process-liveness semantics remain open.
Single/fixed/cached executors now queue real guest work, with Future results,
cancellation, timed worker waits and actual shutdown state. Compiled contracts
and an authored native wait/deliver/cancel flow pass; independent asynchronous
APK workflows remain unproven. [Exact executor profile](threading.md#executors-and-future-results).
A bounded guest layout pass executes inherited APK onMeasure/onLayout callbacks,
including support RecyclerView and DrawerLayout. It renders saved Notepad rows
and moves its closed drawer offscreen; item animations are omitted. XML layout
parameters, attached merge and ViewStub replacement pass compiled checks.
[Guest layout scope and metadata-only drawable limits](framework.md#guest-layout-callbacks-and-xml-metadata).
This does not establish general RecyclerView or AndroidX compatibility.
Bounded GridView binds guest BaseAdapter cells, observer notifications, item-click
callbacks with long IDs, auto-fit columns and four stretch modes. An authored
native fixture verifies image clicks, disabled items and refresh. At most 1,024
cells are materialized; viewport recycling, selection and touch scrolling remain
unsupported. [Grid scope](framework.md#adapter-backed-grids).
String valueOf/toString/length/hashCode/equals/startsWith/contains/
substring/concat/charAt; StringBuilder constructors/append/toString; Integer
parseInt/toString; Float parseFloat (decimal); Double parseDouble/valueOf(D)/doubleValue/toString/isNaN(D); Long toString(J)/rotateRight(JI); Math sqrt/cbrt/sin/cos/tan/log/
exp/abs/pow; Log d/i/w/e. The source's exact signature table is authoritative;
other overloads remain unsupported.
Throwable constructors/getMessage/getLocalizedMessage/getCause/toString,
fillInStackTrace and no-argument printStackTrace retain real DEX locations and
causes for the covered fault paths. Guest diagnostic overrides execute DEX and
their faults propagate. PrintStream/PrintWriter overloads, source-line decoding,
StackTraceElement arrays and suppression remain unsupported. [Diagnostic scope](dex-vm.md).
Android `CharSequence`/`Spanned`/`SpannableStringBuilder` text and a bounded SAX
event parser cover Notepad's rich-text serialization path. DTDs are rejected.
SQLite support includes `SQLiteOpenHelper`, SQL statements/transactions,
`ContentValues` updates, `rawQuery` and typed cursor reads; the pinned Notepad
APK save/restart probe confirms two note rows persist and the reopened list
renders both titles. It also reopens an existing row, updates its title and
multiline plain-text body, refreshes the list and restores both fields in a fresh
editor process. Object-array sort executes stable guest Comparator/Comparable
callbacks. TextUtils UTF-16 search and disjoint plain-text replacement support
this save path. These narrow paths do not imply general text/XML/database
compatibility.
Malformed-body replay separately verifies the original APK logs its SAX fault,
displays its own !ERROR! marker and preserves the raw stored row when opened.
XML metacharacters still do not round-trip through the APK's serializer.
Current source adds explicit same-APK Intent constructors/setClass/setClassName,
startActivity/startActivityForResult, getIntent, action/type/data metadata,
setResult, finish/isFinishing/onBackPressed, Bundle typed extras and back-stack
lifecycle. Authored checks cover result snapshots, cancellation, stopped callers,
GC and callback failures. Native ACTION_OPEN_DOCUMENT_TREE returns the actual
folder choice as a session-local URI; bounded queries and read-only streams are
implemented. [Document and scaling scope](framework.md#read-only-document-trees-and-image-scaling).
Nested Bundle/Parcelable/Parcelable ArrayList state now crosses Activity launches
and results through a bounded Parcel subset. Authored checks exercise actual
guest writers/CREATORs, mutation/GC, state isolation, malformed data and cleanup.
[Exact Parcelable scope](framework.md#bounded-parcelable-state-transfer).
Boxed Integer/Long/Double/Boolean extras and list entries use ordinary Parcel
value tags and preserve shallow-copy identity. General Java serialization remains
unsupported.
The v0.1.0 release predates these additions. [Result and picker scope](framework.md#activity-results-and-native-folder-selection).
Application lifecycle observers add bounded registration/removal, GC-rooted
snapshot delivery from six Activity super methods and canonical getApplication
identity. Native authored navigation/Back/close executes these observers; saved-state
and modern pre/post callbacks remain unsupported. [Lifecycle scope](framework.md).
Platform fragments without Views add queued tag-only transactions and guest
callbacks across navigation, GC and teardown. Duplicate/recursive operations and
callback failures are checked; Views, fragment back stacks and saved state remain
unsupported. [Fragment scope](framework.md#platform-fragments-without-views).
Build.VERSION.SDK_INT exposes a fixed read-only API-21 branch profile, independent
of APK/host metadata. It does not imply complete API-21 support.
[Profile and field checks](framework.md#virtual-api-profile).
SharedPreferences adds String/int/long/float/boolean reads and staged editors,
commit/apply/remove/clear, MODE_PRIVATE stores, Activity.getPreferences and a
minimal application-context singleton. Native authored-note save/restart/clear
and isolated persistence are verified. Apply is synchronous; preference listeners,
String sets and general file APIs remain unsupported. [Storage limits](storage.md).
HashSet/ArrayList/HashMap and basic LinkedHashMap add bounded operations using
guest equals; lists preserve duplicates/order and support indexed operations.
Native HashMap/LinkedHashMap putAll copies bounded entries with snapshot GC roots.
ArrayList/Set iterators
support removal and fail-fast next/remove. Collections.unmodifiableSet/unmodifiableList stay live
and reject mutation. Canonical Class literals work as Map keys, with basic package
metadata. [Collections methods, evidence and ceilings](collections.md).
CopyOnWriteArrayList adds bounded indexed/membership operations and immutable
snapshot iterators: old values survive live changes, GC and serial guest worker
writes. Read-only views preserve this iterator behavior. Reentrant remove equality,
copy constructors, bulk APIs and ListIterator/subList remain unsupported.
LinkedBlockingQueue supports an immediate FIFO subset with fixed/default capacity,
duplicates, null rejection and inherited override dispatch. Worker take/put waits
retain managed frames; timed waits and parallel execution remain unsupported. [Queue limits](collections.md#immediate-fifo-queues).

APK-local Class.forName(String), getName/getClass and no-argument construction
execute guest code. Field opcodes canonicalize inherited declaring owners. Native wrapper TYPE fields
supply canonical primitive Class metadata, with final-write and wrong-kind faults.
[Reflection scope and evidence](reflection.md).

Main Handler/Looper/Message scheduling runs deferred and delayed guest callbacks,
honors dispatch overrides, cancels by identity and retains pending/active payloads
through GC. Native timers and headless manual time are verified in an authored
fixture. Deferred workers execute DEX on a serial shared-heap host executor;
queue/monitor waits, interruption and worker-to-main result posting pass headless
checks. The authored native Start worker action also delivers its main-thread result and closes cleanly. [Exact scheduling methods, clocks and limits](threading.md).
Prepared worker Loopers now route messages through managed callbacks, retain
supported waits and honor quit/quitSafely. Ordering, cancellation, GC, exception
reentry and authored wait/deliver/cancel UI actions pass headless checks. Automatic
native host finish while blocked exits cleanly; manual native Looper button
interaction and independent public-APK Looper workflows remain unverified.

## Known ceilings

- One foreground Activity with a bounded preserved back stack; serial shared-heap VM with bounded guest workers
  and approximate layout/style/configuration. No saved-state recreation/tasks/
  launch modes/general implicit or external Intents. Activity results and native
  ACTION_OPEN_DOCUMENT_TREE are bounded subsets.
- Explicit and common implicit Java exceptions are catchable; unsupported APIs,
  malformed instructions and host resource ceilings remain terminal diagnostics.
- UTF-16 lengths/substrings are honored; isolated surrogates are rejected by Rust.
- Java float string scientific-notation edge cases differ from Rust formatting.
- Failed class initialization is sticky and retains causes; concurrent initialization
  is unsupported. Instruction/field/method checks are not a complete Java type verifier.
- Main blocking waits, blocking native-bridge callbacks/initializers, general wait/notify,
  nested Looper pumps/priority, parallel execution, Timer finalization and JVM process-liveness parity are unsupported.
- Other bulk collections, custom Map copies/views, CopyOnWriteArrayList write revalidation,
  ListIterator/subList, custom class loaders, method/field reflection, general file I/O,
  general SQLite APIs beyond the subset documented in [storage](storage.md),
  bitmap pixel manipulation/Canvas, networking, JNI, JIT,
  APK signature verification, installation registry or Linux native renderer.
- AndroidX, modern Kotlin patterns, Compose, multimedia and games are unsupported.

The catalog calls both public calculators **interactive subsets**, not fully
working. Menus, gestures, complete visual fidelity and all possible numeric
behavior have not been tested. TableLayout/TableRow use the basic linear model;
stretchColumns, gradients and Android themes remain incomplete.
Custom fixtures are labeled separately from independent APK evidence.
