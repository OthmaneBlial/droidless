# Main-thread scheduling and bounded guest workers

Current source owns a bounded main-thread message queue. Handler callbacks execute
the APK's DEX in the same interpreter as lifecycle and input callbacks. Posting
does not run the callback inline. Current source also executes deferred guest
workers with shared objects, independent managed frames and stable Thread identity.
LinkedBlockingQueue take/put can suspend those workers until data/capacity or an
interrupt becomes available. This remains a bounded subset, not full Java concurrency.

## Supported surface

| Family | Exact subset |
|---|---|
| Looper | getMainLooper, myLooper, getThread; prepare creates one metadata-only worker Looper; main quit/quitSafely throw IllegalStateException |
| Handler construction | (), (Looper), (Callback), (Looper, Callback); main Looper only |
| Runnable posts | post(Runnable), postDelayed(Runnable, long), postAtTime(Runnable, long), postAtTime(Runnable, Object, long) |
| Message delivery | obtainMessage(), obtainMessage(int), sendMessage(Message), sendMessageDelayed(Message, long), sendMessageAtTime(Message, long), dispatchMessage(Message), handleMessage(Message) |
| Cancellation | hasCallbacks(Runnable), removeCallbacks(Runnable), removeCallbacks(Runnable, Object), removeCallbacksAndMessages(Object) |
| Message | constructor(), obtain(), getTarget, setTarget, getCallback, getWhen; public what/arg1/arg2/obj fields |
| Clock | SystemClock.uptimeMillis and elapsedRealtime return process-relative monotonic milliseconds |
| Wall clock | System.currentTimeMillis and Date deadlines follow deterministic advancement; native mode reads host wall time |
| Thread | currentThread; constructors (String) and (Runnable, String); getName/setName/getId/isAlive/isDaemon; setDaemon before start; start once, interrupt/isInterrupted/interrupted, holdsLock; explicit run() calls the stored Runnable |
| Timer | constructors (), (boolean), (String), (String, boolean); schedule(TimerTask, long/Date) and repeating long/Date + period overloads; scheduleAtFixedRate repeating long/Date + period overloads; cancel(), purge() |
| TimerTask | constructor(), cancel(), scheduledExecutionTime(); actual APK run() executes on the Timer's guest worker |
| Executors | newSingleThreadExecutor(), newFixedThreadPool(int), newCachedThreadPool(); deferred execute and Callable/Runnable submit overloads |
| ExecutorService | shutdown, shutdownNow, isShutdown, isTerminated, awaitTermination(long, TimeUnit) |
| Future/FutureTask | Callable and (Runnable, result) constructors; run once, get and timed get, cancel, isDone/isCancelled; virtual done and protected set/setException |
| Worker queue waits | LinkedBlockingQueue take()/put(Object); immediate results on main, suspend/resume on a worker |
| Monitors | DEX monitor-enter/exit, reentrant ownership, contention and IllegalMonitorStateException |
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

## Worker execution and limits

Thread.start enqueues execution and returns; it does not call run inline. APK
Thread.run overrides and stored Runnable targets execute their DEX on a named Rust
host worker executor. At event-loop polls the runtime transfers its exclusive
borrow onto that executor, then joins it before main Handler/UI processing.
Guest workers retain distinct Thread identities and frame stacks across polls;
they can migrate between host executor threads. One shared heap is used throughout.
**Guest execution is serial, not parallel CPU execution.** This conservative
scheduler gives sequentially consistent shared memory, with no isolated heap copies.

A poll allows 64 worker slices of 1,024 top-level DEX steps each, shared across
its message dispatches, and at most 64 live workers. Native bridges and class
initialization remain synchronous within a step and share the existing overall
instruction budget. This is not precise wall-time preemption. Blocking workers
execute no more instructions until the queue/monitor is ready. Queue waits retain
the invoke PC and every caller/callee register/result/exception; values and locks
remain GC roots. No empty take or full put is converted into fabricated success.

Monitor ownership is reentrant and belongs to the guest Thread, not the host
executor. Contending workers retain their monitor-enter PC. Interrupt wakes a
queue wait with InterruptedException and clears the interrupt flag; monitor
acquisition remains non-interruptible. Restarting an already started/terminated
Thread throws IllegalThreadStateException. Terminal worker faults remain visible
host diagnostics, mark that worker dead and release its locks. Custom uncaught
exception handlers are not implemented.

Worker View/Activity calls are explicitly rejected before mutation; results must
be posted to a main Handler. Looper.myLooper is null on an unprepared worker;
prepare creates one worker Looper associated with that Thread, and a second prepare
throws RuntimeException. Worker loop()/Handler delivery, priority, sleep/join,
timed queue waits and general worker Looper delivery remain unsupported.
Handler construction on a worker must explicitly select the main Looper.

Blocking on main, or suspension across a synchronous native bridge/initializer,
remains an explicit terminal diagnostic. Immediate take/put work on main. A future
main continuation/native bridge expansion must preserve these waits, not turn
blocking into inline execution. Host close/root finish cancels worker continuations
and releases their roots/locks without claiming guest finally execution at process
shutdown. Main callbacks and widgets retain their host main-thread requirement.

## Executors and Future results

The earlier Executor.execute shortcut ran the Runnable inline and reported fake
shutdown results. Current source removes that shortcut. Single/fixed pools queue
tasks in FIFO order and reuse stable guest Threads, including ThreadLocal values.
Cached pools reuse idle workers, add workers when none are available, and retire
idle workers after 60 seconds on the runtime clock. Factory construction creates
no worker until work is submitted. Guest workers share the serial execution model
and global 64-worker ceiling described above; they do not run in parallel.

Callable submissions retain their actual returned object or original thrown
Throwable. Runnable submissions return null or the supplied result object.
FutureTask runs at most once; its native run path is unwrapped into managed DEX
frames when dispatched by a pool or Thread. Queue, monitor and Future waits retain
those frames and their GC roots. APK run() overrides still execute their bytecode;
suspending through an override's synchronous super.run bridge remains unsupported.
Explicit synchronous FutureTask.run handles nonblocking bodies.

get returns a completed value, throws ExecutionException with the original cause,
or throws CancellationException. Pending worker get suspends; timed get throws
TimeoutException at its preserved deadline. Interrupting a wait throws
InterruptedException and clears the interrupt flag. Null TimeUnit is rejected
even for a completed Future. Waits use monotonic milliseconds; positive
sub-millisecond waits round up. A pending main-thread get remains unsupported;
zero-duration polls and completed gets work on main.

cancel marks the Future done immediately and calls its virtual done override on
the canceling Thread. cancel(false) lets an already running body finish; cancel(true)
also interrupts its runner. Later results do not replace cancellation, and done
is called only once. Normal completion invokes done on the completing worker.
Its UI callbacks must post through the main Handler. Guest task exceptions from
submit are captured as Future failures and the worker remains reusable; uncaught
execute exceptions remain visible and a replacement worker drains remaining work.
Unsupported backend operations remain terminal diagnostics, with an aborted
Future; they are not converted into successful results.

shutdown rejects new submissions and drains accepted work. shutdownNow interrupts
active workers and returns the actual queued Runnable objects; it does not
automatically cancel those returned Futures. isTerminated becomes true only after
shutdown and all queued work/workers end. awaitTermination uses that state, a real
timeout and interruptible worker waiting. Root close drops continuations, cancels
retained pool Futures and removes workers without running guest done/finally.

The profile caps fixed pools at 64 workers and each queue at 16,384 entries;
capacity faults preserve accepted work. Factory results use the native
ThreadPoolExecutor profile, without Java's single-pool wrapper/finalizer.
ThreadFactory overloads, custom pool configuration/rejection handlers,
invokeAll/invokeAny, scheduled executors and runAndReset remain unsupported.
JVM process-liveness/finalization parity is not claimed.

Compiled FutureContract checks worker reuse, fixed-pool progress across blocked
tasks, cached-worker expiry, result/cause retention, wait deadlines/interrupts,
cancellation, shutdown, queued-object identities, failure replacement, GC and
limits. Its portable entry point also passes on desktop Java; this is not an
Android-device differential run. The authored native flow waits for queue input,
delivers Future result: payload through a main Handler, then verifies cancellation
and clean close. No independently completed asynchronous APK workflow is claimed.

## Java timers

Each Timer starts one named guest worker and owns its task queue. Tasks run
sequentially on that worker, with stable Thread identity and ThreadLocal values.
A blocked task retains its managed continuation; later tasks on the same Timer
wait for it. Different timers share the existing serial host executor and its
64-worker/64-slice limits. Timers execute no instructions before a task is due.

One-shot, fixed-delay and fixed-rate schedules support long delays and Date
deadlines. Past fixed-delay Dates become due now; past fixed-rate Dates retain
their original schedule and catch up within the bounded poll budget. Following
API-21 Timer, repeated fixed-delay deadlines use dispatch time before run(), while
fixed-rate deadlines advance from the preceding scheduled deadline.
scheduledExecutionTime exposes the task's most recent scheduled wall-clock time.
Negative delays/Dates, nonpositive periods and initial deadline overflow raise
IllegalArgumentException. Null names/tasks/Dates and task reuse/cancellation raise
the corresponding Java faults.

Timer.cancel discards pending tasks and rejects later scheduling without
interrupting an active task. TimerTask.cancel suppresses future executions and
reports whether the task was still scheduled; purge removes canceled queue
entries and returns their count. An uncaught task fault terminates that Timer,
clears its queue and reports the original diagnostic; other workers still run.
Pending and active tasks are GC roots. Each Timer allows at most 16,384 queued
tasks. Capacity faults leave existing tasks intact and do not consume a rejected
task. Runtime close cancels all timers and releases worker roots and locks.

Worker UI calls remain rejected. A TimerTask must post to the main Handler to
change Views. The authored Scheduling action demonstrates this correct path. In the optimized
native window at 420×720, Start background timer reaches Background timer done: 3;
restarting and canceling leaves Background timer cancelled beyond later deadlines.
Normal close executes teardown and exits 0.
The unmodified SwpieView listener starts its timer on DOWN and cancels on UP;
holding DOWN for 20 seconds instead reaches a TimerTask that calls UI APIs from
the worker and is rejected. Neither behavior establishes a usable slideshow.
[Reproducible public-APK diagnosis](verification.md#current-source-java-timers-and-public-slideshow-diagnosis).

Daemon metadata is retained; independent JVM process-liveness semantics and
Timer finalization on GC are not implemented. Apps must explicitly cancel timers
or close the runtime. Date deadlines are anchored to monotonic uptime when queued;
subsequent host wall-clock jumps are not rebased. Full Android/JVM Timer timing
parity and reference-device differential validation remain open.

```sh
cargo test -p droidless-runtime --test timers --locked
java -cp examples/scheduling/build/classes org.droidless.scheduling.TimerContract
target/release/droidless run --headless --ephemeral fixtures/generated/scheduling.apk \
  --click "Start background timer" --advance-ms 1500 --advance-ms 1500 --advance-ms 1500
# JSON View text: Background timer done: 3
```

Primary references: [API-21 Timer](https://android.googlesource.com/platform/libcore/+/android-5.0.0_r1/luni/src/main/java/java/util/Timer.java),
[API-21 TimerTask](https://android.googlesource.com/platform/libcore/+/android-5.0.0_r1/luni/src/main/java/java/util/TimerTask.java),
[Java Timer](https://docs.oracle.com/javase/8/docs/api/java/util/Timer.html).

## Evidence and reproduction

DEX-to-DEX calls now use a managed continuation stack. The internal evaluator
can pause/resume at instruction boundaries; tests collect between each step and
retain nested call arguments, reference/wide results and caught exceptions.
Native bridges and class initialization remain synchronous within a step.
The worker scheduler now uses this continuation stack for queue/monitor waits.
[Frame semantics and limits](dex-vm.md#frame-and-value-semantics).

The authored [Scheduling fixture](../examples/scheduling/MainActivity.java)
checks deferred execution, equal-time ordering, callback overrides, cancellation
identity, Message fields, main-thread identity, clock boundaries and GC retention.
Rust regressions additionally cover queue capacity, self-posting limits, callback
errors and shutdown. Its separate ThreadContract also passes on desktop Java 17
with Java 8 source/target; it covers metadata/manual run. WorkerContract separately
passes desktop Java and compiled DEX for identity, queue waits, interruption,
reentrant locks and contention. Headless tests verify worker-to-main UI posting,
GC across waits, rejected direct UI access, native-bridge suspension guards, faults,
capacity, bounded spinning and shutdown. The latest native Scheduling check
clicked Start worker and delivered Worker result: kept-consumer:payload in the
native accessibility text, followed by close/teardown and status 0. The long label
clips at the 360×340 viewport; complete layout fidelity is not claimed. No Android
reference differential run or independent asynchronous workflow is claimed.

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

The earlier scheduling and immediate-queue checkpoints diagnosed Notepad's
LinkedBlockingQueue constructor and then Thread.start. Current source resolves
both. Primitive metadata, native map copying and read-only List construction also
resolve. Snapshot CopyOnWriteArrayList construction now resolves too; snapshot
stability across serial worker writes is checked in the authored Collections
fixture. Fixed SDK metadata and Application lifecycle-observer registration now
resolve too. At the earlier scheduling checkpoint, the unchanged Notepad release
stopped at `FileInputStream` in bundled
`com/b/a/a/e.b()` PC `0x0006`, under `e.a()` PC `0x000a` and Application.onCreate
PC `0x0056`, while Stetho reads process information.
The later bounded process-information bridge supersedes that startup blocker.
The current unmodified APK reaches its editor, saves two note titles and displays
both after Back and a fresh process restart; native keyboard entry and saving
are also verified. [Current evidence](verification.md#current-source-public-notepad-editor).
These narrow note flows and authored worker contracts do not establish general
third-party worker compatibility. The 50% checkpoint remains ahead; the v0.1.0
archive predates these runtime increments.

API references: [Android Handler](https://developer.android.com/reference/android/os/Handler),
[Message](https://developer.android.com/reference/android/os/Message),
[Looper](https://developer.android.com/reference/android/os/Looper),
[Java Thread](https://docs.oracle.com/javase/8/docs/api/java/lang/Thread.html),
[LinkedBlockingQueue](https://docs.oracle.com/javase/8/docs/api/java/util/concurrent/LinkedBlockingQueue.html),
[Executors](https://docs.oracle.com/javase/8/docs/api/java/util/concurrent/Executors.html),
[ExecutorService](https://docs.oracle.com/javase/8/docs/api/java/util/concurrent/ExecutorService.html),
[FutureTask](https://docs.oracle.com/javase/8/docs/api/java/util/concurrent/FutureTask.html),
[Java synchronization](https://docs.oracle.com/javase/specs/jls/se8/html/jls-17.html#jls-17.1).

```sh
cargo test -p droidless-runtime --test workers --locked
mkdir -p artifacts/worker-java-contract
javac -source 8 -target 8 -Xlint:-options -d artifacts/worker-java-contract examples/scheduling/WorkerContract.java
java -cp artifacts/worker-java-contract org.droidless.scheduling.WorkerContract
target/release/droidless run --headless --ephemeral fixtures/generated/scheduling.apk --click "Start worker"
# JSON View text: Worker result: kept-consumer:payload
```
