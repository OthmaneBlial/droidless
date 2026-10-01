package org.droidless.scheduling;

import java.util.List;
import java.util.concurrent.*;

/** Portable Java bodies are compiled unchanged into the scheduling APK. */
public class FutureContract {
    static ExecutorService pool;
    static Future<String> first, second;
    static FutureTask<String> direct;
    static LinkedBlockingQueue<String> input;
    static Thread main, firstThread, secondThread, waiter;
    static ThreadLocal<String> local = new ThreadLocal<String>();
    static Throwable cause;
    static volatile int stage, finished, waitState, callbacks, callbackWorker;
    static int mode;
    static String events;
    static Runnable queued;
    static Future<?> runnableFuture;
    static Future<Object> presetFuture;
    static Object preset;
    static int runs;
    static void require(boolean condition) { if (!condition) throw new IllegalStateException("Future contract"); }
    static Callable<String> body() { return new Callable<String>() {
        public String call() throws Exception {
            firstThread = Thread.currentThread();
            require(firstThread != main && firstThread.isAlive() && !firstThread.isDaemon());
            require(firstThread.getName().startsWith("pool-") || firstThread.getName().equals("direct-future"));
            local.set("kept"); stage = 1; System.gc();
            try {
                if (mode == 4) { cause = new IllegalArgumentException("task failure"); throw (IllegalArgumentException) cause; }
                if (mode == 10) Thread.currentThread().getStackTrace(); // Deliberately unsupported API.
                if (mode == 11) Thread.sleep(10);
                return input.take();
            } finally { finished++; }
        }
    }; }
    static Callable<String> next() { return new Callable<String>() {
        public String call() {
            secondThread = Thread.currentThread();
            if (mode == 0 || mode == 4) require(firstThread == secondThread && "kept".equals(local.get()));
            events += "B"; return "second";
        }
    }; }
    public static void prepare(int value) {
        if (pool != null) pool.shutdownNow();
        mode = value; main = Thread.currentThread(); firstThread = secondThread = waiter = null;
        input = new LinkedBlockingQueue<String>(); cause = null;
        stage = finished = waitState = callbacks = callbackWorker = 0; events = "";
        pool = value == 1 ? Executors.newFixedThreadPool(2) : value == 2 ? Executors.newCachedThreadPool() : Executors.newSingleThreadExecutor();
        if (value == 7) {
            direct = new FutureTask<String>(body()); first = direct;
            new Thread(direct, "direct-future").start(); return;
        }
        if (value == 8) {
            direct = new FutureTask<String>(body()) { protected void done() { callbacks++; callbackWorker = Thread.currentThread() == main ? 0 : 1; } };
            first = direct; pool.execute(direct); return;
        }
        if (value == 9) {
            pool.execute(new Runnable() { public void run() { firstThread = Thread.currentThread(); throw new IllegalStateException("execute failure"); } });
            second = pool.submit(next()); first = second; return;
        }
        first = pool.submit(body());
        if (value == 0 || value == 1 || value == 4 || value == 6) second = pool.submit(next());
        if (value == 6) { queued = new Runnable() { public void run() { events += "C"; } }; pool.execute(queued); }
    }
    public static void feed() throws Exception { input.put(new StringBuilder().append("payload").toString()); }
    public static void runNext() { second = pool.submit(next()); }
    public static int state() { return stage; }
    public static int finished() { return finished; }
    public static String events() { return events; }
    public static String firstValue() throws Exception { return first.get(); }
    public static String secondValue() throws Exception { return second.get(); }
    public static String threadName() { return secondThread.getName(); }
    public static int sameThread() { return firstThread == secondThread ? 1 : 0; }
    public static int firstAlive() { return firstThread.isAlive() ? 1 : 0; }
    public static int done() { return first.isDone() ? 1 : 0; }
    public static int cancel(int interrupt) { return first.cancel(interrupt != 0) ? 1 : 0; }
    public static int cancelled() {
        try { first.get(); return 0; } catch (CancellationException expected) { return first.isDone() && first.isCancelled() ? 1 : 0; }
        catch (Exception failure) { throw new IllegalStateException("cancel outcome", failure); }
    }
    public static int failed() {
        try { first.get(); return 0; } catch (ExecutionException expected) { return (mode == 6 ? expected.getCause() instanceof InterruptedException : expected.getCause() == cause) ? 1 : 0; }
        catch (Exception failure) { throw new IllegalStateException("failure outcome", failure); }
    }
    public static void shutdown() { pool.shutdown(); }
    public static int shutdownState() { return pool.isShutdown() ? 1 : 0; }
    public static int terminated() { return pool.isTerminated() ? 1 : 0; }
    public static int zeroAwait() throws Exception { return pool.awaitTermination(0, TimeUnit.MILLISECONDS) ? 1 : 0; }
    public static int reject() {
        try { pool.execute(new Runnable() { public void run() { events += "BAD"; } }); return 0; }
        catch (RejectedExecutionException expected) { return 1; }
    }
    public static int stopNow() {
        List<Runnable> pending = pool.shutdownNow();
        return pending.size() == 2 && pending.get(0) == second && pending.get(1) == queued && !second.isDone() ? 1 : 0;
    }
    public static void waitFor(int kind) {
        waiter = new Thread(new Runnable() { public void run() {
            try {
                waitState = 1;
                if (kind == 2) waitState = pool.awaitTermination(10,TimeUnit.MILLISECONDS) ? 2 : 3;
                else { String value = kind == 0 ? first.get() : kind == 3 ? first.get(1,TimeUnit.NANOSECONDS) : first.get(10,TimeUnit.MILLISECONDS); require("payload".equals(value)); waitState = 2; }
            } catch (TimeoutException expected) { waitState = 3; }
            catch (InterruptedException expected) { require(!Thread.currentThread().isInterrupted()); waitState = 4; }
            catch (Exception failure) { throw new IllegalStateException("waiter failed",failure); }
        }}, "future-waiter"); waiter.start();
    }
    public static int waitState() { return waitState; }
    public static void interruptWaiter() { waiter.interrupt(); }
    public static int callbacks() { return callbacks; }
    public static int callbackWorker() { return callbackWorker; }
    public static void prepareRunnableVariants() {
        pool = Executors.newSingleThreadExecutor(); runs = 0; preset = new Object();
        Runnable body = new Runnable() { public void run() { runs++; System.gc(); } };
        runnableFuture = pool.submit(body); presetFuture = pool.submit(body,preset);
    }
    public static int runnableVariants() throws Exception { return runnableFuture.get() == null && presetFuture.get() == preset && runs == 2 ? 1 : 0; }
    public static int validation() throws Exception {
        int count = 0;
        try { Executors.newFixedThreadPool(0); } catch (IllegalArgumentException expected) { count++; }
        try { new FutureTask<Object>((Callable<Object>)null); } catch (NullPointerException expected) { count++; }
        try { pool.execute(null); } catch (NullPointerException expected) { count++; }
        try { first.get(0,TimeUnit.MILLISECONDS); } catch (TimeoutException expected) { count++; }
        try { first.get(0,null); } catch (NullPointerException expected) { count++; }
        return count;
    }
    public static int completedVariants() throws Exception {
        final Object token = new Object(); final int[] runs = new int[1];
        Runnable body = new Runnable() { public void run() { runs[0]++; System.gc(); } };
        FutureTask<Object> task = new FutureTask<Object>(body,token); task.run(); task.run();
        require(task.get() == token && runs[0] == 1 && !task.cancel(true));
        class Settable extends FutureTask<Object> {
            Settable() { super(body,null); }
            void value(Object value) { set(value); }
            void failure(Throwable value) { setException(value); }
        }
        Settable value = new Settable(); value.value(token); value.run(); require(value.get() == token && runs[0] == 1);
        Settable failure = new Settable(); Throwable cause = new Exception("set failure"); failure.failure(cause);
        try { failure.get(); throw new IllegalStateException("missing failure"); } catch (ExecutionException expected) { require(expected.getCause() == cause); }
        FutureTask<Object> nullTask = new FutureTask<Object>(body,null); nullTask.run(); require(nullTask.get() == null);
        try { task.get(0,null); throw new IllegalStateException("null unit accepted"); } catch (NullPointerException expected) { }
        return runs[0] == 2 ? 1 : 0;
    }
    public static void prepareCapacity() {
        pool = Executors.newSingleThreadExecutor(); Runnable task = new Runnable() { public void run() { } };
        for (int i=0;i<16385;i++) pool.execute(task);
    }
    public static void main(String[] args) throws Exception {
        prepare(0); feed(); require("payload".equals(first.get(2,TimeUnit.SECONDS))); require("second".equals(second.get(2,TimeUnit.SECONDS)));
        require(sameThread() == 1); shutdown(); require(pool.awaitTermination(2,TimeUnit.SECONDS));
        prepare(4); require(failed() == 1); require("second".equals(second.get(2,TimeUnit.SECONDS))); shutdown(); require(pool.awaitTermination(2,TimeUnit.SECONDS));
        require(completedVariants() == 1);
        prepare(8); feed(); require("payload".equals(first.get(2,TimeUnit.SECONDS))); shutdown(); require(pool.awaitTermination(2,TimeUnit.SECONDS));
        require(callbacks == 1 && callbackWorker == 1);
        System.out.println("Future contract passed");
    }
}
