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

`--back` and native Escape dispatch virtual onBackPressed, including APK overrides.
Finishing the last Activity ends the native loop normally; headless output is null.
Limits: 64 Activity instances, 128 pending transitions and 16,384 entries per Bundle.
Saved-state recreation, activity results, launch modes/flags, tasks, implicit/
external intents and launching from non-Activity contexts remain unsupported.

## Preferences subset

Context.getSharedPreferences, Activity.getPreferences and getApplicationContext
support the current lifecycle/storage fixtures. Preference stores support typed
String/int/long/float/boolean values, defaults, contains, staged editors,
remove/clear and commit/apply. Values survive Activity transitions and GC; optional
disk persistence is confined to host-selected per-package directories. Apply is
currently synchronous. MODE_PRIVATE only; listeners/String sets/files/SQLite remain
unsupported. [Exact storage semantics and boundaries](storage.md).

## Java collections subset

HashSet/HashMap provide bounded storage, nulls and guest virtual equals. Set
iteration supports removal and catchable invalid-state/exhaustion/concurrent-change
errors. Collections.unmodifiableSet is a live read-only view, including its
iterator. Class literals have stable identity; Class.getPackage/Package.getName
expose basic metadata. Bulk operations and Map views remain unsupported. [Exact methods and ceilings](collections.md).

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
themes/style inheritance and compact entries are not implemented. Stable public
framework IDs supply OK/Cancel strings; there is no embedded Android resource
installation. String/color/dimension/layout resolution belongs to DROIDLESS.

Binary layouts create TextView, Button, EditText, LinearLayout and FrameLayout.
TableLayout/TableRow use the basic linear model. Attributes include IDs, text,
resource references, width/height, weight, orientation, uniform padding, margins,
text size/color, gravity, enabled/visibility and XML onClick. Recursive include
is bounded. px/dp/sp resolve at density 1. `--size WIDTHxHEIGHT` selects logical host dimensions (128–4096 per axis; default 420×720).

Weighted linear children divide remaining primary-axis space. Measurement is
approximate for explicit weighted base sizes and many Android constraints.
FrameLayout stacks children. Styling is partial; some drawable XML supplies a
flat color, while gradients, ripple/masks, vector/image drawables and unrecognized
styling attributes are omitted. The real screenshot visibly reflects this subset.

## Input/native boundary

NSButton actions invoke the app's View.OnClickListener bytecode; XML onClick
invokes an Activity method. NSTextField changes update EditText, which callbacks
read through getText. Native controls supply focus/selection/accessibility.
The standard macOS Edit menu supplies Cut/Copy/Paste/Select All through AppKit's
responder chain. Native UTF-8 paste into EditText is verified in the preferences fixture.
Escape dispatches Activity Back. Other key-down/up maps digits, letters and common operators to Android KeyEvent codes
and executes OnKeyListener. Initial key focus uses the first enabled visible
listener; complete Android focus/IME/gesture behavior remains future work.

All guest/UI work is currently on the main thread. FFI copies strings synchronously
and retains the host until the event loop ends. Callback errors/panics stop the
loop and report errors. Rust unsafe sites describe their contracts; guest parser/
VM memory uses checked Rust structures, never host pointers.
