# APK-local class lookup and construction

Current source implements `Class.forName(String)`, `Object.getClass`,
`Class.getName`, `Class.getPackage`, `Package.getName` and a subset of
`Class.newInstance`. Class objects have canonical, GC-rooted identity. Lookup
searches the APK's parsed DEX modules and the runtime's known framework subset;
it never loads host Java classes or executes a host JVM as an APK fallback.

Binary names and array names are converted to descriptors. Unknown/invalid names
raise ClassNotFoundException; null raises NullPointerException. Array names are
limited to 255 dimensions and loading an array does not initialize its component.
Ordinary forName initializes the requested class through the guest VM.

No-argument construction executes the APK's own constructor. DEX class and
constructor access flags supply public/package/private checks against the caller.
Abstract/interface/array types and APK classes without a nullary constructor raise
InstantiationException; inaccessible classes/constructors raise IllegalAccessException.
Constructor exceptions propagate without reflective wrapping. Class initialization
retains its existing guest error wrapping, sticky failure and GC-rooted causes.

Field opcodes resolve APK field references to their declaring owner, searching
interfaces before the superclass and bounding the search to 128 visited types.
Inherited aliases share storage; shadowed fields remain separate. Static reads
initialize the declaring owner. Missing fields and static/instance mismatches
raise NoSuchFieldError and IncompatibleClassChangeError.

This is a narrow API subset, not complete reflection or a full Java verifier.
Custom class loaders, external DEX loading, three-argument forName, reflective
method/field invocation, annotation/nest access rules and Class.toString remain
unsupported. Framework construction still requires an implemented constructor.

```sh
cargo test -p droidless-runtime --test reflection --locked
target/release/droidless run --headless --ephemeral fixtures/generated/reflection.apk
```

The authored APK executes ReflectionContract and displays `Reflection passed`.
The same pure-Java contract passes on desktop Java 17 after compiling for Java 8.
It covers lookup, initialization/access faults, unwrapped constructor failures,
arrays, inherited object/int/wide/static fields and shadowing. Rust tests add GC
identity, malformed-name bounds and missing/wrong-kind field regressions.
This is desktop Java differential evidence; no Android reference run is claimed.

Reference contract: [Java 8 Class API](https://docs.oracle.com/javase/8/docs/api/java/lang/Class.html).
