# Register machine, objects and garbage collection

Primary input contracts: [DEX format](https://source.android.com/docs/core/runtime/dex-format)
and [Dalvik bytecode](https://source.android.com/docs/core/runtime/dalvik-bytecode).
Implementation belongs to DROIDLESS, not these reference implementations.

Standalone DEX 035/037/038/039/040 is accepted. Container 041, reverse endian,
optimized/quickened DEX and newer invoke forms are rejected. The parser checks
magic, header/whole-file size, SHA-1, Adler-32, table ranges and referenced IDs.
Class members use delta indexes. Code items retain register counts, instructions,
try ranges/catch handlers and debug offset. Annotations/debug programs are not
fully decoded. This is not a full bytecode verifier.

## Frame and value semantics

One register is a 32-bit primitive word or a typed managed handle. Long/double use
two adjacent low/high words, including in argument lists. Input registers occupy
the final `ins_size` positions in `registers_size`. Each call has its own PC,
register file, invoke result and pending exception. Errors retain method/DEX/PC.

DEX-to-DEX calls now push managed frames into an iterative evaluator rather than
recurse through the Rust interpreter. The caller stays at the invoke PC until
its callee returns; the callee carries the caller's next PC. Returned words become
the caller's rooted invoke result before move-result. Exceptions search handlers
at each faulting/calling PC while unwinding, retaining method/DEX/PC context.

The internal evaluator can stop after an instruction quantum and resume the same
frame stack. Zero steps execute nothing. Compiled tests pause after every DEX step
and collect, checking nested calls, reference/wide returns and caught/uncaught
exceptions. Native bridges and class initialization still execute synchronously
within a step; worker queue/monitor waits now use the managed continuations, but native bridge
suspension and precise wall-time slices remain unsupported.
[Worker boundary](threading.md).

Integer overflow wraps, shifts mask counts, float/double arithmetic uses IEEE
operations and numeric conversions follow the current Rust/Java-compatible
saturation rules. Arithmetic edge tests and compiled Java exercise actual D8
instructions rather than a parallel calculator implementation.

Execution stops after five million instructions per launch/input/close transaction or message poll
or 128 frames. Unsupported opcodes fail rather than acting as successful NOPs.
DEX monitors track reentrant guest Thread ownership, root locked objects, park
contending workers and release locks on unwind. Unbalanced/cross-frame monitor
operations fail explicitly. DEX-declared synchronized flags do not substitute for
monitor instructions; native synchronized methods remain unsupported.
[DEX access flags](https://source.android.com/docs/core/runtime/dex-format).

## Classes and objects

Instances hold descriptor plus fields keyed by declaring class/name/type. Field
opcodes resolve inherited symbolic owners before accessing storage or initializing
the declaring class; missing/wrong-kind fields raise guest linkage errors. Arrays
retain component type and words. Static fields are independent roots, initialized
from encoded values before `<clinit>`. A class is marked initializing to prevent
recursive entry. Failed initialization persists: non-Error guest exceptions are
wrapped in ExceptionInInitializerError, later accesses throw NoClassDefFoundError,
and causes remain GC roots. Existing Error subclasses propagate without wrapping.
The failure rules follow [JLS 12.4.2](https://docs.oracle.com/javase/specs/jls/se25/html/jls-12.html#jls-12.4.2).
Initialization depth is bounded to 128; concurrent initialization is unsupported.
Initialization of default-method superinterfaces is not yet implemented.
Inheritance/virtual/interface dispatch, inherited interface assignability
and reference array covariance are exercised by compiled Java tests. Array reads/
writes validate opcode/component width and reference element assignability.

Explicit throw and host-generated Java faults propagate guest Throwable objects
through the parsed catch handlers in the current frame and callers. Integer/long
zero division, null access/throw, negative array sizes, array bounds/store errors,
casts, string bounds and numeric parse failures are catchable. Typed hierarchy
matching, cross-frame propagation, catch-all/finally and getMessage are tested.
Malformed bytecode, resource ceilings and unsupported APIs remain terminal host
diagnostics; they do not become fake catchable successes.

## Heap

Handles are monotonically allocated and never reused, preventing stale aliases.
Mark/sweep roots include active Activity/content View, statics, interned strings
and frames. Traversal follows object fields, arrays, View children and listeners.
Collection occurs between input callbacks and through explicit System.gc.
Active and paused frame registers, invoke results and pending exceptions remain
roots. No automatic allocation-triggered collector, moving/generational collector
or finalization exists.

Bounds include one million lifetime handles, one million array elements and one
MiB per guest string. These do not enforce overall resident memory: nested arrays
and metadata can still be expensive. `--heap-stats` reports actual object and
collection counts, not inferred peak bytes. Reachability/reclamation tests run
without Android.
