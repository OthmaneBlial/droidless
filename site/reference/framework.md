# Framework, lifecycle, resources and rendering

Binary manifest intent filters identify MAIN/LAUNCHER Activity. Relative component
names and aliases are qualified against the package. The complete parsed XML tree
preserves component attributes, filters, themes and metadata for inspection.
Optional app-defined Application receives construction/onCreate. Main Activity
receives construction, onCreate(null Bundle), onStart and onResume. Returning
from the host loop pauses/stops the foreground Activity and destroys the stack.

Explicit same-APK Intents launch manifest-declared Activity classes. Bundle and
Intent extras support String (including null), int, long, float, double and boolean,
typed defaults, copying, removal and membership. startActivity copies extras;
getExtras returns a copy. Transitions run after the current guest callback returns.
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
Saved-state recreation, activity results, launch modes/flags, tasks, implicit/
external intents and launching from non-Activity contexts remain unsupported.

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
resource references, width/height, weight, orientation, uniform padding, margins,
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
