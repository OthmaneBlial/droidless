# Security boundaries

This prototype is not an audited sandbox. APKs are untrusted binary inputs;
the host process still has the user's permissions. Use trusted APKs.

Compressed/expanded APK totals are capped at 256 MiB, each entry at 64 MiB,
entry count at 50,000. Decompression is bounded independently of declared sizes.
ZIP traversal/duplicates/symlinks are rejected and no APK data is extracted.
Readers check ranges/overflow; XML/layout/encoded-value nesting and reference
chains are bounded. DEX header/digests and referenced table indexes are checked.

Guest references are typed handles, not FFI pointers. Register/array/object
references are checked during execution. Unknown opcodes/classes/framework/native
calls fail explicitly. Frame/instruction/string/array/object limits exist but
are not a complete process memory or CPU-time sandbox.

Guest network, filesystem, clipboard, camera, microphone, location, process and
native-library APIs are unavailable. Inspected manifest permissions grant no host
capabilities. Native controls handle user input, while the AppKit bridge consumes
trusted Rust structures on the main thread. UI strings with NUL fail C conversion;
callback errors stop execution rather than faking success.

No signature trust policy, full verifier, out-of-process isolation, OS sandbox,
adversarial review or continuous coverage-guided fuzz campaign has been completed.
Native codecs/JNI/JIT each require new boundary review before exposure. Future
host I/O must be confined to per-package capabilities; sensitive desktop access
must remain explicit. No such I/O bridge ships in this initial implementation.
