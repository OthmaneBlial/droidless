# Java collections subset

Current source implements bounded guest HashSet, ArrayList and HashMap storage,
plus basic insertion-mode LinkedHashMap and immediate LinkedBlockingQueue operations. Objects stay
in the managed heap; lookups execute the APK's virtual `equals` method. Null keys,
null values and String value equality are supported by the Set/List/Map stores.
Queues reject null elements. There is no host Java runtime.

| Type | Supported methods |
|---|---|
| HashSet | Empty/int-capacity constructors; size, isEmpty, add, contains, remove, clear, iterator |
| ArrayList | Empty/int-capacity constructors; size, isEmpty, add, contains, remove, clear, iterator; indexed get/set/add/remove; indexOf/lastIndexOf |
| HashMap / LinkedHashMap | Empty/int-capacity constructors; size, isEmpty, containsKey, containsValue, get, put, remove, clear; putAll between native maps |
| LinkedBlockingQueue | Empty/fixed-capacity constructors; size, isEmpty, remainingCapacity, add/offer, peek/element, poll/remove, contains, remove(Object), clear; immediate operations and worker take/put waits |
| Set / List iterator | hasNext, next, remove; exhaustion, invalid remove and structural-change exceptions |
| Collections | unmodifiableSet / unmodifiableList: live backing views; supported mutations and iterator.remove throw UnsupportedOperationException |

All stores allow at most 16,384 entries. Negative Set/List/Map initial capacity raises a
catchable IllegalArgumentException. Capacity is a hint; it does not preallocate
guest storage. A single insertion exceeding the limit fails without modifying the store. A bulk
map copy preserves earlier writes if a later entry reaches the limit.
Membership uses linear scans; hash buckets are deferred until profiling justifies
them. Ordering and performance do not reproduce a Java hash-table implementation.
List insertion/removal currently copies the bounded backing vector; Java's
amortized append performance is not reproduced.

## Immediate FIFO queues

LinkedBlockingQueue preserves insertion order and duplicates, with catchable
NullPointerException for null insertion. Constructor capacity must be positive;
the default is Integer.MAX_VALUE. remainingCapacity reports the declared capacity
minus size, independently of the runtime's 16,384-entry resource ceiling. At a
declared full capacity offer returns false and add throws IllegalStateException.
At the host entry ceiling further insertion is a terminal resource diagnostic;
neither failure changes existing data. No capacity-sized allocation is made.

peek/element read the head; poll/remove() remove it. Empty peek/poll return null;
empty element/remove() throw NoSuchElementException. remove(Object) removes only
the first equal value; null membership/removal return false. Membership invokes
the search object's guest equals even for identical references. Inherited add,
element, remove() and isEmpty dispatch virtual offer, peek, poll and size, including
APK overrides. Queue equals/hashCode retain Object identity behavior.

Queue storage reuses the managed collection vector and GC traversal. FIFO updates
copy that bounded vector; a VecDeque can replace it if profiling warrants that.
Structural mutation of this queue during guest equals is explicitly unsupported
and reports a terminal diagnostic after the callback. It does not manufacture a
ConcurrentModificationException or discard the callback's mutations.

Current source also supports take()/put(Object): ready operations execute directly;
empty/full operations retain a worker's managed invocation until data/capacity or
an interrupt is available. Interrupted waits throw InterruptedException and clear
the flag. Shared queue mutation executes serially on the shared guest heap. Main
blocking and suspension across synchronous native bridges remain explicit errors.
Timed offer/poll, iterators, drainTo, bulk/copy operations and queue toString remain
unsupported. Resource limits preserve existing values. The class name does not
imply complete Java concurrency. [Threading boundary](threading.md).

## Sets, lists and maps

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

Other bulk operations, Collection/Map-copy constructors, arrays, ListIterator/subList,
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

## Native map copying and read-only lists

putAll copies between exact native HashMap/LinkedHashMap instances. Guest key
comparison, existing-key replacement, null keys/values, empty and self copies
reuse the existing put path. Null source raises NullPointerException. Native
snapshot roots retain keys and values through guest equality callbacks and GC,
and are released on success or error. Custom Map implementations and subclasses
are rejected before copying; entrySet traversal and eviction hooks remain unsupported.
Source mutation during an equality callback is an explicit terminal diagnostic,
not Java differential evidence. Earlier writes and callback effects remain in place;
bulk copying is not an atomic transaction.

unmodifiableList forwards supported reads to the live backing List, including
APK overrides of get. get/indexOf/lastIndexOf, size/isEmpty/contains and native
iteration are checked. Nested views work. The wrapper preserves RandomAccess
when its backing type has that marker, and retains the backing List through GC.
Indexed and collection mutators raise UnsupportedOperationException before
changing the backing List. Read-only iterators wrap the backing iterator without
changing its mutation permissions: shared cursors still delegate, and a separate mutable alias remains
writable. Iterator.remove rejects mutation through the wrapper; backing
structural changes retain the native iterator's fail-fast behavior. ListIterator,
subList, arrays and collection equals/hashCode/toString remain unsupported.

## Reproduce the compiled conformance check

```sh
cargo test -p droidless-runtime --test collections --locked
target/release/droidless run --headless --ephemeral fixtures/generated/collections.apk
# Desktop Java comparison for the same list/basic LinkedHashMap contract:
mkdir -p artifacts/list-java-contract
javac -source 8 -target 8 -Xlint:-options -d artifacts/list-java-contract examples/collections/ListContract.java
java -cp artifacts/list-java-contract org.droidless.collections.ListContract
# Desktop Java comparison for the same immediate queue contract:
mkdir -p artifacts/queue-java-contract
javac -source 8 -target 8 -Xlint:-options -d artifacts/queue-java-contract examples/collections/QueueContract.java
java -cp artifacts/queue-java-contract org.droidless.collections.QueueContract
# Desktop comparison for native map bulk copying:
mkdir -p artifacts/map-java-contract
javac -source 8 -target 8 -Xlint:-options -d artifacts/map-java-contract examples/collections/MapCopyContract.java
java -cp artifacts/map-java-contract org.droidless.collections.MapCopyContract
```

The authored APK displays `Collections passed` in its headless View snapshot. It
checks guest equality, nulls, duplicates/order, indexed operations, iterator
override dispatch, returns, exceptions, live read-only behavior and class identity.
The same APK now also executes QueueContract: FIFO/duplicates, empty/full behavior,
null rejection, declared capacity, guest equality/faults and inherited override
dispatch. ListContract also checks live/nested read-only List views, virtual reads, nulls,
mutation faults and iterator behavior. MapCopyContract checks native map copies
with guest equality and GC. Rust checks additionally cover snapshot retention/
release, source-mutation diagnostics, partial copies at entry ceilings and rejected
eviction hooks. The same ListContract passes on Java 17 with Java 8 source/target; the
runtime's extra reentrant-mutation guard is checked separately in guest DEX.
QueueContract also passes on Java 17 with Java 8 source/target; its mutation/waiting
limitations are tested separately and are not presented as Java differential evidence.
No Android reference differential run is claimed.
This establishes a generic API subset, not working third-party notes-app UI.

Contract references: [Android ArrayList](https://developer.android.com/reference/java/util/ArrayList),
[Java 8 ArrayList](https://docs.oracle.com/javase/8/docs/api/java/util/ArrayList.html)
and [Java 8 LinkedHashMap](https://docs.oracle.com/javase/8/docs/api/java/util/LinkedHashMap.html).
Queue semantics: [Java 8 LinkedBlockingQueue](https://docs.oracle.com/javase/8/docs/api/java/util/concurrent/LinkedBlockingQueue.html).

Bulk-copy and read-only view contracts: [Java 8 HashMap](https://docs.oracle.com/javase/8/docs/api/java/util/HashMap.html) and [Java 8 Collections](https://docs.oracle.com/javase/8/docs/api/java/util/Collections.html).
