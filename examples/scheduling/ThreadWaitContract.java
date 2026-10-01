package org.droidless.scheduling;

import java.util.concurrent.LinkedBlockingQueue;

/** Portable sleep/join bodies; desktop Java and DROIDLESS have separate drivers. */
public class ThreadWaitContract {
    static Thread worker, joiner, contender;
    static Object lock;
    static LinkedBlockingQueue<String> input;
    static volatile int stage, joined, contended, finished;
    static String output;
    static Runnable onResult;
    static void require(boolean value) { if (!value) throw new IllegalStateException("Thread wait contract"); }
    public static int validate() throws Exception {
        int checked = 0; Thread empty = new Thread();
        try { Thread.sleep(-1); } catch (IllegalArgumentException expected) { checked |= 1; }
        try { Thread.sleep(0, -1); } catch (IllegalArgumentException expected) { checked |= 2; }
        try { Thread.sleep(0, 1000000); } catch (IllegalArgumentException expected) { checked |= 4; }
        try { empty.join(-1); } catch (IllegalArgumentException expected) { checked |= 8; }
        try { empty.join(0, -1); } catch (IllegalArgumentException expected) { checked |= 16; }
        try { empty.join(0, 1000000); } catch (IllegalArgumentException expected) { checked |= 32; }
        Thread.currentThread().interrupt(); empty.join();
        if (Thread.interrupted()) checked |= 64; // No wait on a NEW Thread, so the flag survives.
        Thread.currentThread().interrupt();
        try { Thread.sleep(0); } catch (InterruptedException expected) {
            if (!Thread.currentThread().isInterrupted()) checked |= 128;
        }
        Thread.sleep(0); return checked;
    }
    public static void prepare(final int mode, final int kind, Runnable result) {
        lock = new Object(); input = new LinkedBlockingQueue<String>(); onResult = result;
        stage = joined = contended = finished = 0; output = "";
        worker = new Thread(new Runnable() { public void run() {
            String kept = new StringBuilder().append("kept-").append("sleep").toString();
            try {
                synchronized (lock) {
                    stage = 1; System.gc();
                    if (mode == 0) Thread.sleep(10);
                    else if (mode == 1) Thread.sleep(0, 1);
                    else if (mode == 2) input.take();
                    else if (mode == 3) { Thread.currentThread().interrupt(); Thread.sleep(100); }
                    require(Thread.holdsLock(lock)); output = kept; stage = 2;
                }
            } catch (InterruptedException expected) {
                require(!Thread.currentThread().isInterrupted()); stage = 7;
            } finally { finished++; }
        }}, "sleeper");
        joiner = new Thread(new Runnable() { public void run() {
            joined = 1;
            try {
                if (kind == 1) worker.join(5);
                else if (kind == 2) worker.join(0, 1);
                else if (kind == 3) worker.join(Long.MAX_VALUE, 999999);
                else if (kind == 4) { Thread.currentThread().interrupt(); worker.join(); }
                else if (kind == 5) Thread.currentThread().join(5);
                else worker.join();
                require(kind == 1 || kind == 2 || kind == 5 || !worker.isAlive());
                joined = 2;
                if (onResult != null) onResult.run();
            } catch (InterruptedException expected) {
                require(!Thread.currentThread().isInterrupted()); joined = 7;
            }
        }}, "joiner");
        contender = new Thread(new Runnable() { public void run() {
            synchronized (lock) { require(stage != 1); contended = 1; }
        }}, "contender");
        worker.start(); joiner.start(); contender.start();
    }
    public static void feed() { input.offer("payload"); }
    public static void interruptWorker() { worker.interrupt(); }
    public static void interruptJoiner() { joiner.interrupt(); }
    public static void joinOnMain() throws Exception { worker.join(); }
    public static int joinDead() throws Exception { worker.join(); return worker.isAlive() ? 0 : 1; }
    public static int state() { return stage; }
    public static int joined() { return joined; }
    public static int contended() { return contended; }
    public static int finished() { return finished; }
    public static int alive() { return worker.isAlive() || joiner.isAlive() || contender.isAlive() ? 1 : 0; }
    public static String output() { return output; }
    public static void main(String[] args) throws Exception {
        require(validate() == 255);
        for (int mode : new int[]{0, 1, 3}) {
            prepare(mode, 0, null); worker.join(); joiner.join(); contender.join();
            require(stage == (mode == 3 ? 7 : 2) && joined == 2 && contended == 1 && finished == 1 && alive() == 0);
        }
        prepare(2, 1, null); joiner.join(); require(joined == 2 && worker.isAlive());
        feed(); worker.join(); contender.join(); require(stage == 2);
        prepare(2, 4, null); joiner.join(); require(joined == 7 && worker.isAlive());
        feed(); worker.join(); contender.join();
        System.out.println("Thread wait contract passed");
    }
}
