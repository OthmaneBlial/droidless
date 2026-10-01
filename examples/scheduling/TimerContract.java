package org.droidless.scheduling;

import java.util.Date;
import java.util.Timer;
import java.util.TimerTask;
import java.util.concurrent.LinkedBlockingQueue;

/** Same task bodies execute as compiled DEX in the runtime checks. */
public class TimerContract {
    static Timer timer;
    static TimerTask task;
    static Thread main, seen;
    static ThreadLocal<String> local = new ThreadLocal<String>();
    static LinkedBlockingQueue<String> input;
    static volatile String events;
    static volatile int stage;
    static int cancelled;
    static long origin, firstScheduled, lastScheduled;
    static class Mark extends TimerTask {
        final String letter;
        Mark(String letter) { this.letter = letter; }
        public void run() {
            Thread thread = Thread.currentThread();
            if (thread == main || !"fixture-timer".equals(thread.getName()) || !thread.isDaemon()) throw new IllegalStateException("Timer worker metadata");
            if (seen == null) { seen = thread; local.set("retained"); }
            else if (seen != thread || !"retained".equals(local.get())) throw new IllegalStateException("Timer worker identity/local");
            System.gc();
            if (events.length() == 0) firstScheduled = scheduledExecutionTime();
            lastScheduled = scheduledExecutionTime();
            events += letter;
        }
    }
    public static void prepare(int mode) {
        if (timer != null) timer.cancel();
        timer = new Timer("fixture-timer", true);
        main = Thread.currentThread(); seen = null;
        input = new LinkedBlockingQueue<String>(1);
        events = ""; stage = cancelled = 0; firstScheduled = lastScheduled = 0;
        origin = System.currentTimeMillis();
        task = new Mark("A");
        if (mode == 0) timer.schedule(task, 10);
        else if (mode == 1) timer.schedule(task, 10, 10);
        else if (mode == 2) timer.scheduleAtFixedRate(task, 10, 10);
        else if (mode == 3) timer.scheduleAtFixedRate(task, new Date(origin - 25), 10);
        else if (mode == 4) {
            task = new Mark("A") { public void run() {
                stage = 1;
                try { if (!"payload".equals(input.take())) throw new IllegalStateException("Timer payload"); }
                catch (InterruptedException interrupted) { throw new IllegalStateException("Timer interrupted"); }
                super.run(); stage = 2;
            }};
            timer.schedule(task, 0); timer.schedule(new Mark("B"), 0);
        } else if (mode == 5) {
            timer.schedule(new TimerTask() { public void run() { throw new IllegalStateException("Timer task failure"); }}, 0);
            timer.schedule(task, 0);
        } else if (mode == 6) {
            task = new Mark("A") { public void run() { super.run(); cancelled = cancel() ? 1 : 0; }};
            timer.scheduleAtFixedRate(task, 0, 10);
        } else if (mode == 7) timer.schedule(task, new Date(origin + 10));
        else if (mode == 8) timer.schedule(task, new Date(origin - 25));
        else if (mode == 9) timer.schedule(task, new Date(origin - 25), 10);
        else if (mode == 10) {
            task = new Mark("A") { public void run() { timer.cancel(); super.run(); }};
            timer.schedule(task, 0); timer.schedule(new Mark("B"), 0);
        }
    }
    public static String readEvents() { return events; }
    public static int state() { return stage; }
    public static int cancelResult() { return cancelled; }
    public static long firstOffset() { return firstScheduled - origin; }
    public static long lastOffset() { return lastScheduled - origin; }
    public static long wallElapsed() { return System.currentTimeMillis() - origin; }
    public static int alive() { return seen != null && seen.isAlive() ? 1 : 0; }
    public static void feed() throws InterruptedException { input.put("payload"); }
    public static void cancelTimer() { timer.cancel(); }
    public static int cancelTask() { return task.cancel() ? 1 : 0; }
    public static int purge() { return timer.purge(); }
    public static int rejectReuse() {
        try { timer.schedule(task, 0); return 0; } catch (IllegalStateException expected) { return 1; }
    }
    public static int rejectSchedule() {
        try { timer.schedule(new Mark("X"), 0); return 0; } catch (IllegalStateException expected) { return 1; }
    }
    public static int validation() {
        int checks = 0;
        Timer owner = new Timer(true);
        try {
            try { new Timer((String)null); } catch (NullPointerException expected) { checks++; }
            try { owner.schedule(new Mark("X"), -1); } catch (IllegalArgumentException expected) { checks++; }
            try { owner.schedule(new Mark("X"), 0, 0); } catch (IllegalArgumentException expected) { checks++; }
            try { owner.scheduleAtFixedRate(new Mark("X"), 0, -1); } catch (IllegalArgumentException expected) { checks++; }
            try { owner.schedule(new Mark("X"), Long.MAX_VALUE); } catch (IllegalArgumentException expected) { checks++; }
            try { owner.schedule(new Mark("X"), new Date(-1)); } catch (IllegalArgumentException expected) { checks++; }
            try { owner.schedule(new Mark("X"), (Date)null); } catch (NullPointerException expected) { checks++; }
            try { owner.schedule(null, 0); } catch (NullPointerException expected) { checks++; }
            TimerTask unused = new Mark("X");
            if (!unused.cancel() && !unused.cancel()) checks++;
            try { owner.schedule(unused, 0); } catch (IllegalStateException expected) { checks++; }
            owner.cancel(); owner.cancel();
            try { owner.schedule(new Mark("X"), 0); } catch (IllegalStateException expected) { checks++; }
        } finally { owner.cancel(); }
        return checks;
    }
    public static void prepareCapacity() {
        prepare(0);
        for (int i = 1; i <= 16384; i++) timer.schedule(new Mark("bounded"), 100000);
    }
    public static void refillCapacity() {
        task.cancel();
        if (timer.purge() != 1) throw new IllegalStateException("capacity purge");
        timer.schedule(new Mark("replacement"), 100000);
    }
    public static void main(String[] args) throws Exception {
        if (validation() != 11) throw new IllegalStateException("Timer validation");
        prepare(4);
        feed();
        long deadline = System.currentTimeMillis() + 2000;
        while (!"AB".equals(events) && System.currentTimeMillis() < deadline) Thread.sleep(1);
        timer.cancel();
        if (!"AB".equals(events) || stage != 2 || rejectReuse() != 1 || rejectSchedule() != 1) throw new IllegalStateException("serial Timer tasks");
        System.out.println("Timer JVM contract passed: validation, serial tasks, worker identity, cancellation");
    }
}
