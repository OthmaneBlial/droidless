package org.droidless.scheduling;

import java.util.ArrayList;
import java.util.concurrent.LinkedBlockingQueue;

/** Ordinary Java worker/queue/monitor contract; desktop Java is a separate reference. */
public class WorkerContract {
    static LinkedBlockingQueue<String> input, output;
    static Thread worker, second, main;
    static Object lock;
    static volatile int stage, finished, locked;
    static Runnable callback;
    static String awaitInput() throws InterruptedException { return input.take(); }
    static class Consumer extends Thread {
        Consumer() { super("consumer"); }
        public void run() {
            try {
                if (Thread.currentThread() != this || Thread.currentThread() == main || !isAlive()) throw new IllegalStateException("worker identity");
                synchronized (lock) {
                    synchronized (lock) { if (!Thread.holdsLock(lock)) throw new IllegalStateException("reentrant lock"); }
                    if (!Thread.holdsLock(lock)) throw new IllegalStateException("outer lock");
                }
                String local = new StringBuilder().append("kept-").append(getName()).toString();
                stage = 1;
                String item = awaitInput();
                output.put(local + ":" + item);
                stage = 2;
                if (callback != null) callback.run();
                awaitInput();
                stage = 3;
            } catch (InterruptedException expected) {
                if (isInterrupted()) throw new IllegalStateException("interrupt flag not cleared");
                stage = 7;
            } finally { finished++; }
        }
    }
    static class WaitingEquals {
        public boolean equals(Object other) {
            try { input.take(); return true; }
            catch (InterruptedException interrupted) { return false; }
        }
    }
    public static void prepare(int mode, Runnable onResult) {
        input = new LinkedBlockingQueue<String>(1); output = new LinkedBlockingQueue<String>(1);
        lock = new Object(); stage = finished = locked = 0;
        callback = onResult; main = Thread.currentThread(); second = null;
        if (mode == 0) worker = new Consumer();
        else worker = new Thread(new Runnable() { public void run() {
            try {
                if (Thread.currentThread() != worker || Thread.currentThread() == main) throw new IllegalStateException("Runnable identity");
                if (mode == 1) {
                    output.put("first"); stage = 1; output.put("second"); stage = 2;
                } else if (mode == 2) {
                    synchronized (lock) { locked++; output.put("locked"); input.take(); locked--; }
                } else if (mode == 3) {
                    synchronized (lock) { throw new IllegalStateException("worker failure"); }
                } else if (mode == 4) {
                    callback.run();
                } else if (mode == 5) {
                    synchronized (lock) {
                        ArrayList<Object> values = new ArrayList<Object>(); values.add(new Object());
                        values.contains(new WaitingEquals());
                    }
                } else if (mode == 6) {
                    while (true) stage++;
                }
            } catch (InterruptedException expected) { stage = 7; }
            finally { finished++; }
        }}, "producer");
        worker.start();
        if (mode == 2) {
            second = new Thread(new Runnable() { public void run() {
                synchronized (lock) {
                    if (locked != 0 || !Thread.holdsLock(lock)) throw new IllegalStateException("monitor contention");
                    locked++; locked--; stage = 2;
                }
            }}, "contender");
            second.start();
        }
    }
    public static void feed() throws InterruptedException { input.put("payload"); }
    public static String pollOutput() { return output.poll(); }
    public static void interrupt() { worker.interrupt(); }
    public static int state() { return stage; }
    public static int finished() { return finished; }
    public static int alive() { return worker.isAlive() ? 1 : 0; }
    public static int canAcquire() { synchronized (lock) { return Thread.holdsLock(lock) ? 1 : 0; } }
    public static void prepareCapacity() {
        for (int i=0; i<65; i++) new Thread("bounded").start();
    }
    public static int restartRejected() {
        try { worker.start(); return 0; } catch (IllegalThreadStateException expected) { return 1; }
    }
    public static void main(String[] args) throws Exception {
        prepare(0, null); feed();
        if (!"kept-consumer:payload".equals(output.take())) throw new IllegalStateException("queue/local result");
        interrupt(); worker.join();
        if (stage != 7 || finished != 1 || alive() != 0 || restartRejected() != 1) throw new IllegalStateException("termination");
        prepare(1, null);
        if (!"first".equals(output.take()) || !"second".equals(output.take())) throw new IllegalStateException("FIFO put");
        worker.join();
        if (stage != 2 || finished != 1) throw new IllegalStateException("producer");
        prepare(2, null);
        if (!"locked".equals(output.take())) throw new IllegalStateException("holder");
        feed(); worker.join(); second.join();
        if (stage != 2 || locked != 0) throw new IllegalStateException("monitor");
        System.out.println("Worker contract passed");
    }
}
