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

## Dependencies

`zip` handles ZIP/deflate; `sha1` checks DEX signatures; `anyhow` retains errors;
`serde`/`serde_json` export inspection/test data; build-only `cc` compiles AppKit.
None supplies Android execution, lifecycle, resources or the View model.
Cargo.lock fixes the dependency versions.
