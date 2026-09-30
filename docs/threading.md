# Main-thread scheduling and Java thread metadata

Current source owns a bounded main-thread message queue. Handler callbacks execute
the APK's DEX in the same interpreter as lifecycle and input callbacks. Posting
does not run the callback inline. This is a main Looper subset; background guest
execution and blocking waits remain unsupported. LinkedBlockingQueue now has an
[immediate FIFO subset](collections.md#immediate-fifo-queues), without take/put or waiting.

## Supported surface

| Family | Exact subset |
|---|---|
| Looper | getMainLooper, myLooper, getThread; quit/quitSafely throw IllegalStateException for the main Looper |
| Handler construction | (), (Looper), (Callback), (Looper, Callback); main Looper only |
| Runnable posts | post(Runnable), postDelayed(Runnable, long), postAtTime(Runnable, long), postAtTime(Runnable, Object, long) |
| Message delivery | obtainMessage(), obtainMessage(int), sendMessage(Message), sendMessageDelayed(Message, long), sendMessageAtTime(Message, long), dispatchMessage(Message), handleMessage(Message) |
| Cancellation | hasCallbacks(Runnable), removeCallbacks(Runnable), removeCallbacks(Runnable, Object), removeCallbacksAndMessages(Object) |
| Message | constructor(), obtain(), getTarget, setTarget, getCallback, getWhen; public what/arg1/arg2/obj fields |
| Clock | SystemClock.uptimeMillis and elapsedRealtime return process-relative monotonic milliseconds |
| Thread metadata | currentThread; constructors (String) and (Runnable, String); getName/setName/getId/isAlive; explicit run() calls the stored Runnable |
| Collection | System.gc invokes the runtime's managed collector |

Handler dispatch honors APK overrides. Default dispatch runs a posted Runnable
first; otherwise it calls Handler.Callback and, unless handled, the virtual
handleMessage override. Callbacks may enqueue later callbacks. Cancellation is
scoped to the receiving Handler and matches callbacks/tokens by object identity;
a null cancellation token matches all tokens. Pending and active Messages are
GC roots, retaining their Handler, callback and payload graph.

Messages run in deadline order, then FIFO insertion order for equal deadlines.
Negative delays become zero; past absolute deadlines are due at the next poll.
Each Message is one-shot: queued or consumed Messages cannot be sent again.
Dispatch/cancellation clears payloads and marks the Message consumed. Public
pooling/recycle and other obtain overloads are unsupported.

## Host clocks and shutdown

Headless execution starts at zero and advances only through `--advance-ms` or
Runtime.advance_time. Immediate posts drain after launch and each replay action.
Native AppKit execution uses Rust Instant and polls at event-loop boundaries,
with an idle wait of up to 50 ms. It redraws after callbacks mutate guest Views.
This does not model Android device boot time, deep sleep or precise timer latency.
Switching to the native clock disables manual advancement.

There are at most 16,384 pending Messages and 1,024 callbacks per poll, sharing
the existing five-million-instruction budget. Clock/deadline arithmetic is
checked. Limits report terminal diagnostics and preserve remaining queued work.
Uncaught callback exceptions propagate with their original guest cause and clean
interpreter frames. Root Activity finish or host close cancels pending work;
subsequent posts return false. Finishing a child Activity retains the app queue.

All execution remains serial on the main thread. The main Thread object has
stable identity, and unstarted Threads have names and unique IDs. Calling run()
manually is ordinary synchronous execution. **Thread.start fails explicitly**;
it does not substitute a synchronous run. Thread priority, sleep/join/interrupt,
worker Loopers, HandlerThread, synchronization barriers, java.util.Timer,
executors and blocking queue waits are outside this subset.

## Evidence and reproduction

The authored [Scheduling fixture](../examples/scheduling/MainActivity.java)
checks deferred execution, equal-time ordering, callback overrides, cancellation
identity, Message fields, main-thread identity, clock boundaries and GC retention.
Rust regressions additionally cover queue capacity, self-posting limits, callback
errors and shutdown. Its separate ThreadContract also passes on desktop Java 17
with Java 8 source/target; this covers metadata/manual run only. No Android
reference differential run or independent asynchronous APK workflow is claimed.

```sh
cargo build --release --locked
target/release/droidless run --headless --ephemeral fixtures/generated/scheduling.apk \
  --click "Start timer" --advance-ms 1500 --advance-ms 1500 --advance-ms 1500
# JSON View text: Timer done: 3
target/release/droidless run --ephemeral --size 360x340 fixtures/generated/scheduling.apk
```

Actual native clicks produced Waiting for timer, then Timer done: 3 after three
delayed callbacks. Starting and cancelling before the first deadline left Timer
cancelled after that deadline. Finish later executed its delayed finish and
emitted onPause/onStop/onDestroy, exiting with status 0.

![Actual authored Scheduling APK after three native timer callbacks](assets/scheduling-native.png)

At this scheduling checkpoint the unchanged Notepad v1.0.0 release passed
Thread(String) construction and stopped at LinkedBlockingQueue construction.
The immediate-queue increment now resolves that constructor and stops at
Thread.start, in DBFlow's `f/b/a/b.a()` at PC `0x0007`, under Application.onCreate
PC `0x0016`. It still reaches no Activity/UI or notes workflow. Blocking transactions
and real worker execution are the next diagnosed requirements. The 50% checkpoint remains
ahead; the v0.1.0 release archive predates this scheduling support.

API references: [Android Handler](https://developer.android.com/reference/android/os/Handler),
[Message](https://developer.android.com/reference/android/os/Message),
[Looper](https://developer.android.com/reference/android/os/Looper),
[Java Thread](https://docs.oracle.com/javase/8/docs/api/java/lang/Thread.html).
