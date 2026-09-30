# Framework, lifecycle, resources and rendering

Binary manifest intent filters identify MAIN/LAUNCHER Activity. Relative component
names and aliases are qualified against the package. The complete parsed XML tree
preserves component attributes, filters, themes and metadata for inspection.
Optional app-defined Application receives construction/onCreate. Main Activity
receives construction, onCreate(null Bundle), onStart and onResume. Returning
from the host loop calls onPause/onStop/onDestroy. A back stack is not implemented.

Exact method signatures map to DROIDLESS behavior. setContentView accepts a View
or layout resource; findViewById searches the guest graph. Widget mutations alter
objects read by rendering. Window metrics use the logical host dimensions and
density 1. Unknown classes/methods/opcodes and native methods fail with diagnostics.
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
is bounded. px/dp/sp resolve at density 1.

Weighted linear children divide remaining primary-axis space. Measurement is
approximate for explicit weighted base sizes and many Android constraints.
FrameLayout stacks children. Styling is partial; some drawable XML supplies a
flat color, while gradients, ripple/masks, vector/image drawables and unrecognized
styling attributes are omitted. The real screenshot visibly reflects this subset.

## Input/native boundary

NSButton actions invoke the app's View.OnClickListener bytecode; XML onClick
invokes an Activity method. NSTextField changes update EditText, which callbacks
read through getText. Native controls supply focus/selection/accessibility.
Key-down/up maps digits, letters and common operators to Android KeyEvent codes
and executes OnKeyListener. Initial key focus uses the first enabled visible
listener; complete Android focus/IME/gesture behavior remains future work.

All guest/UI work is currently on the main thread. FFI copies strings synchronously
and retains the host until the event loop ends. Callback errors/panics stop the
loop and report errors. Rust unsafe sites describe their contracts; guest parser/
VM memory uses checked Rust structures, never host pointers.
