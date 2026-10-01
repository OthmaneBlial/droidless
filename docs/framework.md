# Framework, lifecycle, resources and rendering

Binary manifest intent filters identify MAIN/LAUNCHER Activity. Relative component
names and aliases are qualified against the package. The complete parsed XML tree
preserves component attributes, filters, themes and metadata for inspection.
Optional app-defined Application receives construction/onCreate. Main Activity
receives construction, onCreate(null Bundle), onStart and onResume. Returning
from the host loop pauses/stops the foreground Activity and destroys the stack.

Explicit same-APK Intents launch manifest-declared Activity classes. Bundle and
Intent extras support String (including null), int, long, float, double and boolean,
typed defaults, copying, removal and membership. Nested Bundles, Parcelables and
Parcelable ArrayLists are supported. startActivity snapshots supported extras
through Parcel; getExtras and Bundle/Intent copy constructors make shallow Bundle
copies. Transitions run after the current guest callback returns.
The previous Activity pauses, the next creates/starts/resumes, then the previous
stops. finish/Back resumes the preserved parent through restart/start/resume and
stops/destroys the outgoing Activity. Each instance retains its title, content
and Intent, and remains a GC root. Finishing a stopped Activity destroys it without
changing the foreground screen. isFinishing remains true during teardown.

Application.registerActivityLifecycleCallbacks / unregisterActivityLifecycleCallbacks
use a per-Application bounded managed ArrayList. Registrations preserve order,
duplicates and nulls; removal uses the first guest-equal entry. Activity.getApplication
and getApplicationContext share the package's canonical Application. The native
Activity super implementations deliver created/started/resumed/paused/stopped/
destroyed observers at the super call, including navigation, Back and native close.
They do not send a second notification after an APK override returns.

Each delivery takes a GC-rooted snapshot: registration/removal during a callback
affects later events, while the current snapshot still runs in order. Callback
faults propagate and temporary roots are released. Registered Screen objects and
active navigation actions stay rooted across callbacks and GC. The registry uses
the existing 16,384-entry List ceiling and its equality-mutation guard. Callback
delivery is synchronous on main; worker/native-bridge waits remain unsupported.
Saved-state delivery/restoration, modern pre/post observers and missing-super
enforcement remain unsupported. No Android reference run is claimed.
[Application registry reference](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-5.0.0_r1/core/java/android/app/Application.java)
and [Activity super-call reference](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-5.0.0_r1/core/java/android/app/Activity.java).

`--back` and native Escape dispatch virtual onBackPressed, including APK overrides.
Finishing the last Activity ends the native loop normally; headless output is null.
Limits: 64 Activity instances, 128 pending transitions and 16,384 entries per Bundle.
Saved-state recreation, launch modes/flags, tasks, general implicit/external
intents and launching from non-Activity contexts remain unsupported.

## Activity results and native folder selection

startActivityForResult preserves request codes and the actual calling Activity;
negative request codes launch without a result. setResult retains its code and
optional Intent, and finish snapshots that Intent and its Bundle extras. Later
mutations cannot change the queued result. Back defaults to RESULT_CANCELED.
Results arrive in virtual guest onActivityResult before the caller resumes;
stopped callers retain pending results until they return. Finished callers are
ignored. Payloads and callers stay rooted across GC; callback failures release
temporary roots and still destroy the outgoing Activity.

Launcher Intents carry MAIN and the explicit component. Intent action/type/data
constructors, getters and setters support null clearing, action interning and
the mutual exclusion of setData/setType. Copy construction isolates Bundle extras.
View.getWindowToken returns a stable managed IBinder identity for attached Views;
isAttachedToWindow follows the same hierarchy. Detached Views return null/false.
The token models window attachment only; Binder IPC remains unsupported.

The one supported implicit action is ACTION_OPEN_DOCUMENT_TREE from an Activity
with a nonnegative request code. macOS opens a real NSOpenPanel folder chooser,
pauses the caller, and returns cancellation or an opaque content URI through the
same result path. A host-selected directory is retained as a session-local
directory capability; URI.parse alone grants nothing. At most 64 tree grants and
one pending chooser are supported. Headless execution reports the need for native
selection explicitly. Bounded document queries and read-only streams now use
these grants; writes and persistent grants remain unsupported. [Security boundaries](security.md).

References: [API-21 Intent](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-5.0.0_r1/core/java/android/content/Intent.java),
[ActivityThread result delivery](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-5.0.0_r1/core/java/android/app/ActivityThread.java).

## Bounded Parcelable state transfer

Parcel.obtain/recycle, byte positions/size/available data, int/long/float/double,
nullable UTF-16 Strings, write/readParcelable and write/readBundle form a bounded
API-21 subset. Supported Bundle values are primitive values, Strings, nested
Bundles, Parcelables and ArrayLists containing supported reference values/nulls.
The buffer is limited to 4 MiB, nesting to 32 and maps/lists to 16,384 entries.
Malformed lengths, magic, tags, truncation, invalid positions, unpaired surrogates,
cycles and recycled use fail explicitly.

Guest Parcelable.writeToParcel and the APK's static CREATOR.createFromParcel run
in DEX; ClassLoaderCreator callbacks receive the allowed loader token. Native Uri
uses its String representation. Class.getClassLoader returns null for framework
classes and a canonical opaque APK loader identity for APK classes. This identity
cannot load host classes, files or external DEX. Start and finish result snapshots
reconstruct their extras; they do not share mutable Parcelable objects with the
sender. Bundle/Intent copy constructors retain Android's shallow object values.
GC roots protect snapshot elements across mutating writers and CREATOR callbacks;
errors release roots and restore recursion/read bounds.

Integer, Long, Double and Boolean extras also retain Serializable marker identity
and typed getter values. Bundle primitive writes box these four types immediately,
preserving object identity across shallow copies. Parcel transport uses Android's
ordinary primitive value tags, including boxed entries in supported ArrayLists;
it does not implement Java object serialization. Float/Byte/Short/Character
boxing and arbitrary Serializable object transport remain unsupported. The
compiled BoxedExtras contract checks aliases, nulls, wrong-type defaults, wide
values, mixed lists, GC and explicit custom-serialization failure.

The authored Parcels APK verifies callback execution, Unicode/wide values, null
list elements, shallow versus transported state, source-list mutation during GC,
activity results and callback failures. It is not an Android-device differential
test. Binder/file descriptors, binary marshalling APIs, typed arrays/lists,
general Java serialization, custom loaders and full reflection access rules
remain unsupported. Unsupported values produce diagnostics instead of aliasing
the sender's objects.

References: [API-21 Parcel](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-5.0.0_r1/core/java/android/os/Parcel.java),
[BaseBundle](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-5.0.0_r1/core/java/android/os/BaseBundle.java).

## Read-only document trees and image scaling

Context.getContentResolver is canonical within a runtime. DocumentsContract tree/
document ID extraction and URI builders preserve encoded path segments; Uri path
segments are read-only Lists. These helpers grant no I/O. Only the runtime's own
`content://droidless.documents` provider accepts access, and only for an actually
selected session tree and its descendants.

ContentResolver.query supports document metadata or immediate children, with
`document_id`, `mime_type`, `_display_name`, `_size` and read-only `flags` columns.
Null projection returns these five columns; explicit projection preserves order.
Snapshots use the existing Cursor implementation and deterministic UTF-16 filename
order. Selection arguments, custom sort orders, cancellation signals, other
providers and unsupported columns fail explicitly. Enumeration is capped at
4,096 entries and paths at 64 segments; non-UTF-8 names and malformed URI escapes
are rejected. Symlinks, special files and hard-linked files are omitted from
child enumeration and rejected for direct stream access.

openInputStream uses the granted directory descriptor, opens every intermediate
directory without following links and validates the final file descriptor. Streams
snapshot at most 64 MiB of a regular, single-link file and reuse existing guest
read/skip/available/close and BitmapFactory decoding. Both declared size and actual
read length are bounded. No ambient `file://` or arbitrary guest path is exposed.
Like API-21 DocumentsProvider, stream opening extracts the document ID even from
a URI with a children suffix. [Security scope](security.md).

Natural Collections.sort reuses the stable bounded List sort and virtual guest
Comparable callbacks, including null-comparator sort. Snapshot elements remain
rooted across guest GC/mutation; failures release temporary roots. String.lastIndexOf
uses UTF-16 positions for String and character overloads, including supplementary
code points, individual surrogate units and negative search bounds.

Object-array `Arrays.sort` adds natural/Comparator and range overloads through the
same stable sort. Guest callbacks, GC, faults and unsupported concurrent mutation
are checked. [Array sorting limits](collections.md#object-array-sorting).

TextUtils.indexOf(CharSequence, CharSequence) uses UTF-16 positions for supported
String/builder/spanned values. TextUtils.replace supports disjoint nonempty plain
text patterns: it replaces each pattern's first original match, so newly inserted
text is not searched again. Existing/destination spans, overlapping patterns,
empty patterns and unequal arrays fail explicitly. Replacement arrays are bounded
at 16,384 elements and output at 1 MiB. General guest CharSequence implementations
and a complete Editable replacement engine remain unsupported.
Reference: [API-21 TextUtils](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-5.0.0_r1/core/java/android/text/TextUtils.java).

ImageView.ScaleType has canonical enum values and retained get/set state.
FIT_XY, FIT_START/CENTER/END, CENTER, CENTER_CROP and CENTER_INSIDE render through
AppKit with clipping and per-side padding; XML scaleType is also retained. MATRIX
and custom image matrices remain unsupported. The unmodified SwpieView APK now
queries and decodes three selected-folder PNG/JPEG/WebP thumbnails and renders
them natively with CENTER_CROP. Thumbnail selection now transfers the image stack
through actual guest Parcel callbacks, opens its full-screen Activity and displays
the selected image. Native Escape returns to all three thumbnails. ImageView's
setImageURI decodes granted document streams and closes them even on decode failure;
null clears the displayed bitmap. Other URI providers remain unsupported.

SwpieView's static-image viewing and Back flow are verified. A host replay now
delivers actual touch events to its GestureDetector: horizontal swipes run the
APK's onFling callback, select next/previous images and respect both boundaries.
Confirmed taps run its hide/show callbacks and restore the original controls,
also verified with actual mouse taps in the optimized native window. Native drag
verification remains pending because the UI automation tool could not locate the
window for its coordinate drag; swipe evidence is host replay only.
GIF animation, slideshow, lifecycle auto-hide and full Android styling remain open.

FrameLayout now honors XML layout_gravity and explicit LayoutParams.gravity for
top/center/bottom and left/center/right positions, including per-side padding and
the retained margins. START/END follow the profile's default left-to-right
direction; RTL, foreground padding and full Android measurement remain open.
This moves SwpieView's bottom controls below its toolbar and lets root touch
dispatch reach the slideshow listener. An ordinary DOWN/UP starts then cancels
the APK's timer; a held DOWN executes its background task and rejects direct UI
access. [Timer semantics and limits](threading.md#java-timers).

Reference: [API-21 FrameLayout](https://android.googlesource.com/platform/frameworks/base/+/android-5.0.0_r1/core/java/android/widget/FrameLayout.java).

## Single-pointer touch and gestures

MotionEvent supports DOWN/UP/MOVE/CANCEL, copies, recycling, local/raw coordinates,
offset/setLocation, action/pointer metadata and monotonic down/event times. Pointer
ID/index 0 is the supported profile; multi-pointer actions, invalid coordinates,
recycled events and invalid/out-of-order streams fail explicitly.

Activity and View dispatch call actual guest overrides and OnTouchListeners.
ViewGroups hit-test children in reverse order, translate child coordinates and
retain the DOWN target through release or cancellation. Unconsumed clickable
Views share performClick with the existing host click path. Disabled controls
suppress listeners; dragging outside cancels the default click. Native mouse
down/drag/up enter this path when the app registers touch behavior, and losing
window focus sends CANCEL. Editable AppKit controls retain their native focus and
selection path; full Android text touch/focus and ViewGroup interception are not
implemented.

GestureDetector owns event snapshots and listener references through collection.
Show press (100 ms), long press (600 ms after DOWN), confirmed single tap (300 ms)
and double tap use the existing Handler queue. Movement beyond 8 logical pixels
produces scroll callbacks and cancels tap/press timers. Fling uses a bounded
20-sample, 100-ms linear velocity estimate, clamped to 8,000 pixels/second with a
50-pixel/second threshold; it does not claim Android VelocityTracker parity.
SimpleOnGestureListener provides Android's default callback bodies. Failed
callbacks cancel pending gesture timers and release temporary roots.

VelocityTracker.obtain, addMovement, computeCurrentVelocity, X/Y getters, clear
and recycle reuse that bounded 100-ms/20-sample linear estimator. Units and
maximum speed are honored, duplicate timestamps replace the latest sample,
recycled input fails, and movement is copied independently of MotionEvent
lifetime. Only pointer 0 is tracked; other IDs return zero. This is approximate
velocity support, not Android's native fitting algorithm.

UNSPECIFIED View measurement now leaves intrinsic sizes unbounded instead of
clipping them to a zero spec size. MATCH_PARENT contributes intrinsic size in
that case; EXACTLY and AT_MOST still constrain it. FrameLayout measurement takes
the maximum child extent on both axes. This fixes zero-height Notepad cards;
full Android measurement and scrolling remain incomplete.
Reference: [VelocityTracker](https://developer.android.com/reference/android/view/VelocityTracker).

The base ViewGroup/ViewParent onStartNestedScroll callback returns false, as on
Android; guest overrides still execute from DEX. Accepted nested scrolling is
not implemented. Guest layout now positions the closed Notepad drawer offscreen;
native painting also excludes descendants of invisible ancestors.

View translation and alpha reach native geometry/opacity. System UI flags are
retained as app state; the desktop content profile has zero Android system-bar
insets. requestApplyInsets requests layout, without full WindowInsets dispatch.
No Android system bars, timed property animation, touch history or multi-touch
are claimed. [API-21 GestureDetector reference](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-5.0.0_r1/core/java/android/view/GestureDetector.java).

References: [API-21 DocumentsContract](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-5.0.0_r1/core/java/android/provider/DocumentsContract.java),
[DocumentsProvider](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-5.0.0_r1/core/java/android/provider/DocumentsProvider.java),
[SwpieView image code](https://github.com/err4nt/SwpieView/blob/d371afbe8337c6244e3ef0a41415b08b14f1c88c/app/src/main/java/org/voidptr/swpieview/ImageContainer.java).

## Platform fragments without Views

Activity.getFragmentManager retains one managed manager per Activity. Tag-only
FragmentTransaction.add queues additions until executePendingTransactions, an
Activity transition, or the next host event boundary. Guest Fragment callbacks
run from attach/create through activity-created/start/resume, pause/stop and
destroy-view/destroy/detach. Activity creation completes before activity-created;
downward fragment callbacks precede the Activity callback, following the
[API-21 lifecycle order](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-5.0.0_r1/core/java/android/app/Activity.java).
Arguments, tag lookup, Activity/manager identity and added/resumed state are retained.
Queued transactions and callback snapshots stay rooted through guest GC; dispatch
faults release temporary roots and the execution guard. Duplicate commit/add,
tag changes, active argument changes and recursive execution raise guest errors.

Limits: 64 attached fragments, 64 additions per transaction, 128 queued transactions
and 1,024 dispatch batches. Fragment methods run on main. Returned Views, container
mounting, remove/replace, fragment back stacks, children, saved-state recreation
and missing-super enforcement remain unsupported. This is a platform Fragment
subset, not general support-library or AndroidX Fragment compatibility. SwpieView
attaches and creates its bundled ReportFragment before its next startup blocker.

## Application metadata and widget state

Context.getApplicationInfo retains a canonical managed ApplicationInfo shared by
Activity and Application contexts. Package name, qualified application name,
resource label/icon IDs and targetSdkVersion come from the APK manifest; a missing
target defaults to minSdkVersion, then 1. PackageManager.getActivityInfo receives
an independent copy. This metadata is separate from the fixed API-21 runtime
profile. UID, paths, flags and other ApplicationInfo fields remain incomplete.

AnimatorListenerAdapter implements both listener interfaces and its six empty
API-21 defaults; APK callback overrides still execute in DEX. This does not add
animation event delivery. OverScroller's timed-scroll mode uses the shared
monotonic runtime clock, Android's default viscous curve or a guest Interpolator,
Java float rounding, final positions, forceFinished and abortAnimation. Guest
interpolator failures propagate and temporary roots are released. Fling/springback
physics remain unsupported; single-pointer gesture delivery is described above.

View.setBackground invokes virtual setBackgroundDrawable, preserving APK
overrides and retained Drawable identity, including null clearing. Rendering is
still limited to the existing flat-color/raster subset. Content descriptions
retain supported String/Spanned values through GC, preserve empty strings, compare
through virtual equals, and clear on null. Nonempty labels promote automatic
accessibility importance. XML labels and AppKit accessibility labels are wired;
accessibility events, general guest CharSequence implementations and full
accessibility-node behavior remain incomplete.

References: [API-21 adapter](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-5.0.0_r1/core/java/android/animation/AnimatorListenerAdapter.java),
[OverScroller](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-5.0.0_r1/core/java/android/widget/OverScroller.java),
[default interpolator](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-5.0.0_r1/core/java/android/widget/Scroller.java),
[View](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-5.0.0_r1/core/java/android/view/View.java).

## Adapter-backed grids

GridView constructors, XML column/spacing/stretch attributes and programmatic
geometry setters feed a two-dimensional native layout. AUTO_FIT and the four
stretch modes retain requested values separately from measured values.
BaseAdapter cells are created by real guest getView calls; getCount, view types,
isEnabled and 64-bit getItemId also dispatch into the APK. Item clicks reach the
actual OnItemClickListener. AppKit image/container clicks use the same runtime
callback path, preserving the host's own gesture recognizers.

DataSetObservable uses the existing managed Observable registry with reverse
live-list delivery, including self-removal and guest GC. Adapter replacement
unregisters the previous observer and clears its children. Changed/invalidated
notifications trigger deferred rebinding; binding faults retain the previous
cells and release temporary roots. Public child mutations are rejected because
the adapter owns the children. Native XML constructors now run, and unfinished
inflation trees stay rooted across guest constructors/class initialization.
Basic LayoutParams width, height and LinearLayout weight are read live.

Limits: 1,024 materialized cells, 256 view types and 32 nested binding passes.
getView receives null convertView; viewport recycling, selection, touch scrolling,
complete Android measurement and advanced LayoutParams remain unsupported.
Adapter changes during binding are explicitly rejected. These are authored
headless/native contracts; SwpieView now also completes folder selection, static-image viewing and Back. Its host gesture replay verifies swipes and confirmed taps; slideshow remains open.

References: [API-21 GridView](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-5.0.0_r1/core/java/android/widget/GridView.java),
[BaseAdapter](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-5.0.0_r1/core/java/android/widget/BaseAdapter.java),
[DataSetObservable](https://github.com/aosp-mirror/platform_frameworks_base/blob/android-5.0.0_r1/core/java/android/database/DataSetObservable.java).

## Preferences subset

Context.getSharedPreferences, Activity.getPreferences and getApplicationContext
support the current lifecycle/storage fixtures. Preference stores support typed
String/int/long/float/boolean values, defaults, contains, staged editors,
remove/clear and commit/apply. Values survive Activity transitions and GC; optional
disk persistence is confined to host-selected per-package directories. Apply is
currently synchronous. MODE_PRIVATE only; listeners/String sets/general files
remain unsupported. A separate SQLite method subset supports the Notepad save
path. [Exact storage semantics and boundaries](storage.md).

## Java collections subset

HashSet/ArrayList/HashMap and basic LinkedHashMap operations provide bounded
storage, nulls and guest virtual equals. Lists preserve duplicates/order and add
indexed reads, updates, insertion/removal and first/last index lookup. ArrayList/Set
iteration supports removal and catchable invalid-state/exhaustion/concurrent-change
errors. Collections.unmodifiableSet is a live read-only view, including its
iterator. Class literals have stable identity; Class.getPackage/Package.getName
expose basic metadata. Native map copying and live read-only List views are also
checked. CopyOnWriteArrayList supports snapshot iteration across live changes and
GC; reentrant write equality remains unsupported. Other bulk operations, Map
views, access-order maps and eviction
hooks remain unsupported. [Exact methods and ceilings](collections.md).

LinkedBlockingQueue adds immediate FIFO add/offer, head reads/removal, size/capacity,
membership and clear, with guest equality and inherited override dispatch. Queue
take/put waits run on the bounded serial guest worker executor. Timed waits and
weakly consistent queue iterators remain unsupported.
[Queue semantics and limits](collections.md#immediate-fifo-queues).

## Main-thread scheduling subset

Handler/Looper/Message queue deferred and delayed guest callbacks, with identity
cancellation, virtual dispatch and GC retention. Native callbacks update Views;
headless `--advance-ms` provides deterministic replay. Minimal Thread metadata
and explicit manual run are supported. Deferred Thread.start executes DEX workers,
with bounded queue/monitor waits, interruption and main Handler result delivery.
Single/fixed/cached executor factories queue work on reusable guest workers.
Callable/Runnable submissions and FutureTask retain actual values/causes, support
cancellation and worker get/deadline waits, and report real shutdown/termination
state. [Executor methods and limits](threading.md#executors-and-future-results).
Main waits, native bridge/initializer suspension and parallel CPU execution remain
unsupported. [Exact methods, clocks, limits and native evidence](threading.md).

## Virtual API profile

Build.VERSION.SDK_INT is a read-only native static int with a fixed value of 21.
This is the runtime's API-branch profile, independent of host OS, APK min/target SDK
or package name. It is metadata for app version checks, not complete Android API-21
support. APK redefinition of the native VERSION class is rejected, so guest class
definitions cannot replace the profile. Unknown methods and fields still fail explicitly. Other Build/version
fields, selectable profiles and complete Android configuration remain unsupported.

Native SDK reads, repeated reads through GC, Class lookup and inherited aliases
are checked in compiled DEX. Field aliases resolve to the declaring native owner
without initializing a subclass. Writes raise IllegalAccessError; instance access
raises IncompatibleClassChangeError, and mismatched types raise NoSuchFieldError.
This framework-specific check is not part of the desktop Java differential run.
[Android field contract](https://developer.android.com/reference/android/os/Build.VERSION#SDK_INT)
and [API-21 version code](https://developer.android.com/reference/android/os/Build.VERSION_CODES#LOLLIPOP).

## APK classes and Java numbers

APK-local Class lookup and no-argument reflective construction execute guest
initializers/constructors with access and exception checks. Inherited field
references resolve to their declaring owner. [Methods and limits](reflection.md).
Double.valueOf(D), doubleValue, instance/static toString and isNaN(D), plus
Long.toString(J), support the new public calculator. Raw double bits, including
negative zero and NaN payloads, survive boxing/unboxing. Other numeric wrapper
methods remain unsupported; Double equals/hashCode fail explicitly.

Exact method signatures map to DROIDLESS behavior. setContentView accepts a View
or layout resource; findViewById searches the guest graph. Widget mutations alter
objects read by rendering. Window metrics use the logical host dimensions and
density 1. `--size WIDTHxHEIGHT` selects logical host dimensions (128–4096 per axis; default 420×720). Unknown classes/methods/opcodes and native methods fail with diagnostics.
There is no APK-specific mathematical output or emulator fallback.

## Resource/layout subset

resources.arsc parsing covers package/type/key pools, simple typed values,
reference chains and map entries, including sparse/16-bit offsets. Default
configuration wins; otherwise the first variant is used. Qualifier matching,
full theme resolution and compact entries are not implemented. Supported style
bags merge explicit and implicit parents, with cycle/depth checks. Stable public
framework IDs supply OK/Cancel strings; there is no embedded Android resource
installation. String/color/dimension/layout resolution belongs to DROIDLESS.

Binary layouts create TextView, Button, EditText, LinearLayout and FrameLayout.
TableLayout/TableRow use the basic linear model. Attributes include IDs, text,
resource references, width/height, weight, orientation, per-side padding, margins,
text size/color, image `src`/`srcCompat`, gravity, enabled/visibility and XML
onClick. Recursive include is bounded. px/dp/sp resolve at density 1.
`--size WIDTHxHEIGHT` selects logical host dimensions (128–4096 per axis; default 420×720).

`Resources.getXml` opens packaged binary XML as an `XmlResourceParser` cursor.
The current subset covers document/tag/text events, namespaces, depth and line
numbers, named and indexed attributes, `nextTag`, `nextText`, and `close`.
Binary XML resource maps retain attribute IDs. Resources, Theme and Context
attribute arrays overlay explicit XML values on supported styles and convert
float/dimension values. The authored APK checks all three entry points.
TextView.setTextAppearance applies inherited size and flat color; typography
and stateful text colors remain incomplete. Paint retains graphics style/stroke
configuration. LayoutTransition retains stagger/parent settings and ViewGroup
ownership; native layout changes remain immediate without animation callbacks.

`String.hashCode` uses Java UTF-16 value hashing. Decimal Float/Double parsing
accepts Java whitespace, suffixes and special values; hexadecimal literals fail
explicitly. A Matrix subset implements identity/copy, values, concatenation,
translation/scale/rotation and in-place point/vector mapping in
[Android multiplication order](https://developer.android.com/reference/android/graphics/Matrix).
Bounded Path move/line/quadratic/cubic/close commands are retained, and graphics
enum constants have canonical identity and Java Enum metadata. This lets the
unchanged SwpieView APK inflate its bundled vector test in guest DEX; rendering
vectors through Canvas remains unsupported.

Weighted linear children divide remaining primary-axis space. Measurement is
approximate for explicit weighted base sizes and many Android constraints.
FrameLayout stacks children. Packaged PNG/JPEG/WebP resources flow through
`BitmapFactory.decodeResource`, `decodeStream` and `decodeByteArray`; the supported
`Options` subset is bounds metadata and `inSampleSize`. `Bitmap.getWidth`/
`getHeight` and `ImageView.setImageBitmap`, `setImageResource` and
`setImageDrawable` feed AppKit `NSImageView` rendering. Each encoded image is
bounded to 32 million pixels. The native image fixture verifies all three formats.
Bitmap pixel operations, `Canvas`, animated/vector drawables, gradients,
ripple/masks and many Android layout/style constraints remain unsupported.
The real screenshot visibly reflects this subset.

## Input/native boundary

NSButton actions invoke the app's View.OnClickListener bytecode; XML onClick
invokes an Activity method. NSTextField changes update EditText, which callbacks
read through getText. Native controls supply focus/selection/accessibility.
The standard macOS Edit menu supplies Cut/Copy/Paste/Select All through AppKit's
responder chain. Native UTF-8 paste into EditText is verified in the preferences fixture.
Escape dispatches Activity Back. Other key-down/up maps digits, letters and common operators to Android KeyEvent codes
and executes OnKeyListener. Initial key focus uses the first enabled visible
listener; complete Android focus/IME/gesture behavior remains future work.

UI work stays on main; bounded DEX workers execute serially on the shared-heap host
executor. FFI copies strings synchronously
and retains the host until the event loop ends. Callback errors/panics stop the
loop and report errors. Rust unsafe sites describe their contracts; guest parser/
VM memory uses checked Rust structures, never host pointers.

## Guest layout callbacks and XML metadata

Before rendering and root touch DOWN, layout_snapshot invokes inherited,
APK-defined onMeasure/onLayout callbacks with current parent-relative bounds.
Children laid out by a parent are not called twice; requestLayout and changed
bounds allow another pass. Translation is excluded from stored layout bounds
and added once when rendering. Invalid input is rejected before layout and
callback failures unwind temporary roots. This bounded pass rebuilds the View
tree between callbacks; it is not a full Android ViewRoot or incremental layout
engine. Native viewport resizing and property animation remain incomplete.

TextView measurement now retains an owned Layout with text, available text width
and calculated line count. getLayout is null before measurement and after text,
size, padding or line-policy changes; an unchanged measurement retains identity.
Explicit newlines, narrow widths, word breaks and empty text contribute real
lines, using the existing approximate advance of 0.6 times text size per Unicode
scalar. The same line count drives intrinsic height; max/min lines constrain the
View height without changing the underlying Layout count. EditText's covered
Editable.append(CharSequence) path updates its owner and invalidates measurement.
Compiled checks exercise inherited onMeasure, GC, failure recovery and retained
old text snapshots. This is an approximate plain-text profile, not Android font
shaping, bidi, styled metrics, ellipsizing or native multiline painting parity.
The null-before-measurement behavior follows the [TextView contract](https://developer.android.com/reference/android/widget/TextView#getLayout()).

Inflation retains XML AttributeSet and Context, calls the actual parent's
virtual generateLayoutParams, and attaches children incrementally. Native base,
margin, frame, linear and table parameters retain sizes, margins, gravity and
weight; legal TableRow dimension defaults are accepted. Attached merge layouts
return their real parent; unattached merges fail. ViewStub inflates its XML
resource, replaces itself at the same index and keeps the original parameters
and inflated ID. Visibility is forwarded to the replacement. Indexed child removal retains parent links, guest hierarchy callbacks and index
faults. Transient state is reference-counted and group queries include descendants;
parent transient-state notification callbacks remain incomplete. Programmatic
stub configuration and weak-reference collection parity remain unsupported.

The compiled CustomLayout contract checks those callbacks and root touch,
collection/failure recovery, per-side padding, measure specs/state bits, suggested
minimum sizes, LTR/RTL Gravity resolution, canonical Rect/RectF fields,
point containment, intersections and translated matrices. Window system-UI
visibility is zero in the unconfigured desktop profile. Legacy fitSystemWindows
applies padding and consumes a Rect only when fits-system-windows is enabled;
WindowInsets/listener dispatch remains unsupported. Android font metrics remain approximate;
TextView baseline uses the current approximate ascent. Relative margin getters
use physical-edge fallback; relative-margin resolution is incomplete. Elevation
and getZ retain guest state with zero translationZ; native shadow/Z-order rendering
and custom child drawing-order configuration are unsupported.
Unfocused groups return null from getFocusedChild. Guest requestFocus is still
unsupported; native editor focus is not mirrored as Android focus state.

CheckedTextView retains checked state. Drawable state uses current enabled and
pressed flags, checked additions, virtual guest callbacks, duplicate-parent
state and capacity-aware merging. Focus/window/selection/activation lifecycles
are incomplete. Flat colors and bounded resource selectors expose genuine
ordered positive/negative state matching, default colors and statefulness.
TextView XML theme color references use the APK's existing theme styles; native
text currently paints the default palette entry. Selector alpha/theme item
values and dynamic native state colors remain unsupported. Compound drawable
slots and tint values retain managed identity/metadata; their icon/checkmark/tint
painting is not implemented. No general Android styling parity is claimed.

SparseArray/SparseIntArray indexOfKey returns ordered signed-key ranks and
complemented insertion points. Collections.reverse uses virtual List get/set,
retaining guest overrides, read-only faults and nonstructural iterator behavior.
Math.min(float,float) retains Java NaN and signed-zero behavior.

AppKit hit testing converts mouse positions to the content view's superview
coordinates; guest MotionEvents retain flipped content coordinates. This keeps
editable native controls on their focus/text-selection route.
[NSView hitTest contract](https://developer.apple.com/documentation/appkit/nsview/hittest(_:)).
