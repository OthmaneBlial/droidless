# Java collections subset

Current source implements bounded guest HashSet, ArrayList and HashMap storage,
plus the inherited basic operations of insertion-mode LinkedHashMap. Objects stay
in the managed heap; lookups execute the APK's virtual `equals` method. Null keys,
null values and String value equality are supported. There is no host Java runtime.

| Type | Supported methods |
|---|---|
| HashSet | Empty/int-capacity constructors; size, isEmpty, add, contains, remove, clear, iterator |
| ArrayList | Empty/int-capacity constructors; size, isEmpty, add, contains, remove, clear, iterator; indexed get/set/add/remove; indexOf/lastIndexOf |
| HashMap / LinkedHashMap | Empty/int-capacity constructors; size, isEmpty, containsKey, containsValue, get, put, remove, clear |
| Set / List iterator | hasNext, next, remove; exhaustion, invalid remove and structural-change exceptions |
| Collections | unmodifiableSet: live backing view; mutation and iterator.remove throw UnsupportedOperationException |

All stores allow at most 16,384 entries. Negative initial capacity raises a
catchable IllegalArgumentException. Capacity is a hint; it does not preallocate
guest storage. New entries exceeding the limit fail without modifying the store.
Membership uses linear scans; hash buckets are deferred until profiling justifies
them. Ordering and performance do not reproduce a Java hash-table implementation.
List insertion/removal currently copies the bounded backing vector; Java's
amortized append performance is not reproduced.

ArrayList preserves insertion order, duplicates and nulls. Indexed get/set/remove
accept 0 through size-1; insertion additionally accepts size. Bad indexes raise
catchable IndexOutOfBoundsException without changing the list. set returns the old
element and preserves the structural version. Object removal removes the first
matching element; indexOf/lastIndexOf execute the search object's guest equals.
List searches call guest equals even for identical references and observe a
callback's nonstructural replacement of later elements. Hash-backed stores keep
their identity shortcut.

Iterator removal updates its cursor and expected modification version. Other
structural changes invalidate next/remove; hasNext is not a fail-fast check.
Adding an existing Set member and replacing an existing Map value preserve the
version. Structural changes from a guest equals callback are detected before
using a saved index. List iterator removal dispatches an APK subclass's remove(int)
override. Iterators, wrappers, keys and values retain their GC roots.

Bulk operations, Collection/Map-copy constructors, arrays, ListIterator/subList,
Map views/iterators, cloning, serialization and collection equals/hashCode/toString
remain unsupported. LinkedHashMap load-factor/access-order constructors and
subclasses/eviction hooks are rejected; inherited put does not silently skip a
guest removeEldestEntry override. Unmodifiable iteration currently requires this
runtime's native collection iterator. Unknown calls fail explicitly.
Class literals now have stable identity, so repeated class references work as
Map keys; Class.getPackage and Package.getName provide basic package metadata.
APK-local class lookup and no-argument construction have a separate
[reflection subset](reflection.md). Custom loaders and method/field reflection
remain unsupported.

## Reproduce the compiled conformance check

```sh
cargo test -p droidless-runtime --test collections --locked
target/release/droidless run --headless --ephemeral fixtures/generated/collections.apk
# Desktop Java comparison for the same list/basic LinkedHashMap contract:
mkdir -p artifacts/list-java-contract
javac -source 8 -target 8 -Xlint:-options -d artifacts/list-java-contract examples/collections/ListContract.java
java -cp artifacts/list-java-contract org.droidless.collections.ListContract
```

The authored APK displays `Collections passed` in its headless View snapshot. It
checks guest equality, nulls, duplicates/order, indexed operations, iterator
override dispatch, returns, exceptions, live read-only behavior and class identity.
Rust checks additionally cover GC retention, entry ceilings and rejected eviction
hooks. The same ListContract passes on Java 17 with Java 8 source/target; the
runtime's extra reentrant-mutation guard is checked separately in guest DEX.
No Android reference differential run is claimed.
This establishes a generic API subset, not working third-party notes-app UI.

Contract references: [Android ArrayList](https://developer.android.com/reference/java/util/ArrayList),
[Java 8 ArrayList](https://docs.oracle.com/javase/8/docs/api/java/util/ArrayList.html)
and [Java 8 LinkedHashMap](https://docs.oracle.com/javase/8/docs/api/java/util/LinkedHashMap.html).
