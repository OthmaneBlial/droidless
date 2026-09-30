# Architecture and decisions

Three crates keep ownership clear: `droidless-formats` parses immutable APK/DEX/
XML/resource metadata; `droidless-runtime` implements execution, heap, framework
and Views; `droidless` implements CLI and the AppKit bridge. Neither parser nor VM
depends on AppKit or an Android SDK. APKs are read without extraction.

DEX IDs resolve per module; signatures and class definitions resolve across
modules. Duplicate classes are rejected. Each method call owns a Dalvik register
frame. APK-defined methods run before framework compatibility methods. Virtual
calls follow receiver ancestry; direct/static/super calls use the declared class.
The native API table matches exact descriptors/signatures, not APK package names.
The calculator's control flow/state/operators remain DEX instructions; Java Math
calls use host math as a peripheral facility.

Guest Views live in the same managed heap as app objects. XML inflation and
programmatic constructors produce the same View model. Measurement exports logical
rectangles; NSButton/NSTextField render them. Native actions carry integer guest
handles back to Rust and execute the original listeners. Guest references never
become pointers. Rust FFI sites document synchronous lifetime/thread contracts.

## ADR 001: interpret before compiling

A bounded register interpreter is the semantic reference. Defer JIT/IR and foreign
native translation until compatibility and profiling justify them. No ART/Dalvik
library or emulator can substitute for DROIDLESS execution.

## ADR 002: AppKit first

Development is on macOS ARM64. Native controls provide accessibility, focus and
input without a browser. A small Objective-C C-ABI shim owns the window; Rust owns
APK behavior. JSON View snapshots keep tests independent of a desktop session.
Linux native rendering is a separate future backend, not an unverified claim.

## ADR 003: local-only CI

User instruction overrides the original GitHub CI request. `tools/ci.sh` runs
fmt/check/test/clippy/release locally. No Actions workflow is installed and source
repository Actions are disabled. Website deployment is separate from source CI.

## ADR 004: managed DEX call continuations

DEX calls use a managed frame stack and iterative evaluator. Each callee carries
its return PC; exception unwinding keeps the caller at the invoke PC. Instruction
slices retain the same registers/results/exceptions and existing GC traversal.
This removes Rust recursion from DEX-to-DEX calls and supplies the continuation
needed for future waiting workers. Native bridge callbacks and class initialization
remain synchronous. Bounded guest workers now use these continuations for queue
and monitor waits; suspension across a native bridge/initializer stays unsupported.
Do not replace those waits with inline execution or clone isolated app heaps.

## Dependencies

`zip` handles ZIP/deflate; `sha1` checks DEX signatures; `anyhow` retains errors;
`serde`/`serde_json` export inspection/test data; build-only `cc` compiles AppKit.
`cap-std`/`cap-fs-ext` supply directory-relative, no-follow host I/O for isolated
preferences; `serde_json` also stores the bounded DROIDLESS preference format.
None supplies Android execution, lifecycle, resources or the View model.
Cargo.lock fixes the dependency versions.

## ADR 005: shared-heap serial worker executor

Workers have their own managed frame stack and stable guest Thread identity.
A Rust scoped host executor receives the exclusive Runtime borrow for bounded
slices; main Handler/UI execution resumes after it joins. Logical workers can
migrate across executor threads while all guest objects remain in one heap.
Serial execution avoids shared mutable heap races and makes deterministic tests
possible. Parallel CPU execution and a persistent executor are deferred; the
scope and current blocking/native bridge limits are explicit in [threading.md](threading.md).

## ADR 006: a fixed virtual API profile

Build.VERSION.SDK_INT reads 21 for every APK and host. The initial branch profile
uses the API-21 baseline of the authored fixtures; it does not measure framework
coverage or promise a complete Android 5.0 environment. Do not vary it to bypass
an app's startup code or derive it from the APK's target SDK. Native final-field
checks and rejection of guest VERSION redefinition keep APK code from replacing
the profile. Other system metadata and
selectable profiles will follow real execution needs with consistent values and
branch-specific tests. [Exact current surface](framework.md#virtual-api-profile).
