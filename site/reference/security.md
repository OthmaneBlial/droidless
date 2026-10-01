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

Private preferences and SQLite use a host-selected apps root with directory
capabilities per validated package. No-follow directory/file
access, package identity checks, case-distinct preference filenames, Unix hard-link
rejection, bounded JSON and atomic writes have regressions. Damaged files fail
without reset. See [storage behavior and limits](storage.md).
ACTION_OPEN_DOCUMENT_TREE is a separate explicit host choice: the native folder
chooser retains a directory descriptor for the selected tree and returns an
opaque content URI. Grants last only for the current runtime session, with a
64-grant ceiling. URI parsing grants no access. Read-only metadata queries and
streams stay confined to the selected tree's open directory descriptor. Traversal,
foreign/unknown grants, symlinks, special files and hard-linked files are rejected;
unsafe child entries are omitted from enumeration. Intermediate directories and
final files are opened without following links. Streams verify the opened file
and bound declared and actual length to 64 MiB. Enumeration is capped at 4,096
entries and path depth at 64. Grants disappear on runtime close/restart; writes,
persistent permissions, other providers and ambient file URIs remain unavailable.

General file, network, clipboard, camera, microphone, location, process and
native-library APIs are unavailable. Inspected manifest permissions grant no host
capabilities. Native controls handle explicit user input/paste, while the AppKit bridge consumes
trusted Rust structures on the main thread. UI strings with NUL fail C conversion;
callback errors stop execution rather than faking success.

No signature trust policy, full verifier, out-of-process isolation, OS sandbox,
adversarial review or continuous coverage-guided fuzz campaign has been completed.
Native codecs/JNI/JIT each require new boundary review before exposure. Future
host I/O must be confined to per-package capabilities; sensitive desktop access
must remain explicit. Package-name impersonation, host modification of data and
concurrent processes are outside the current storage trust model.
