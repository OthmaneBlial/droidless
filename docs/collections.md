# Java collections subset

Current source implements bounded guest HashSet and HashMap storage. Objects stay
in the managed heap; lookups execute the APK's virtual `equals` method. Null keys,
null values and String value equality are supported. There is no host Java runtime.

| Type | Supported methods |
|---|---|
| HashSet | Empty/int-capacity constructors; size, isEmpty, add, contains, remove, clear, iterator |
| HashMap | Empty/int-capacity constructors; size, isEmpty, containsKey, containsValue, get, put, remove, clear |
| Set iterator | hasNext, next, remove; exhaustion, invalid remove and structural-change exceptions |
| Collections | unmodifiableSet: live backing view; mutation and iterator.remove throw UnsupportedOperationException |

Both stores allow at most 16,384 entries. Negative initial capacity raises a
catchable IllegalArgumentException. Capacity is a hint; it does not preallocate
guest storage. New entries exceeding the limit fail without modifying the store.
Membership uses linear scans; hash buckets are deferred until profiling justifies
them. Ordering and performance do not reproduce a Java hash-table implementation.

Iterator removal updates its cursor and expected modification version. Other
structural changes invalidate next/remove; hasNext is not a fail-fast check.
Adding an existing Set member and replacing an existing Map value preserve the
version. Structural changes from a guest equals callback are detected before
using a saved index. Iterators, wrappers, keys and values retain their GC roots.

Bulk operations, arrays, Map views/iterators, cloning, serialization and collection
equals/hashCode/toString remain unsupported. Unmodifiable iteration currently
requires this runtime's native Set iterator. Unknown calls fail explicitly.
Class literals now have stable identity, so repeated class references work as
Map keys; Class.getPackage and Package.getName provide basic package metadata.
APK-local class lookup and no-argument construction have a separate
[reflection subset](reflection.md). Custom loaders and method/field reflection
remain unsupported.

## Reproduce the compiled conformance check

```sh
cargo test -p droidless-runtime --test collections --locked
target/release/droidless run --headless --ephemeral fixtures/generated/collections.apk
```

The authored APK displays `Collections passed` in its headless View snapshot. It
checks guest equality, nulls, returns, exceptions, live read-only behavior and
class identity. Rust checks additionally cover GC retention and entry ceilings.
This establishes a generic API subset, not working third-party notes-app UI.
