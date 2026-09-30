package org.droidless.collections;

import java.io.Serializable;
import java.util.AbstractQueue;
import java.util.Collection;
import java.util.NoSuchElementException;
import java.util.Queue;
import java.util.concurrent.BlockingQueue;
import java.util.concurrent.LinkedBlockingQueue;

/** Immediate queue semantics, run unchanged on desktop Java and in guest DEX. */
public class QueueContract {
    static int equalityCalls;
    static BlockingQueue<Object> retained;
    static LinkedBlockingQueue<Object> mutating;
    static class Key {
        int value;
        Key(int value) { this.value = value; }
        public boolean equals(Object other) {
            equalityCalls++; System.gc();
            return other instanceof Key && ((Key) other).value == value;
        }
    }
    static class SelfFalse {
        public boolean equals(Object other) { equalityCalls++; return false; }
    }
    static class ThrowingKey {
        public boolean equals(Object other) { throw new IllegalStateException("queue equality failure"); }
    }
    static class MutatingKey {
        public boolean equals(Object other) { mutating.clear(); return false; }
    }
    static class RejectingQueue extends LinkedBlockingQueue<Object> {
        int offers;
        public boolean offer(Object value) { offers++; return false; }
    }
    static class OverridingQueue extends LinkedBlockingQueue<Object> {
        int peeks; int polls; int sizes;
        public Object peek() { peeks++; return "guest peek"; }
        public Object poll() { polls++; return "guest poll"; }
        public int size() { sizes++; return 1; }
    }
    static void check(boolean condition, String message) {
        if (!condition) throw new IllegalStateException(message);
    }
    public static int contract() {
        try { new LinkedBlockingQueue<Object>(0); return 0; } catch (IllegalArgumentException expected) {}
        try { new LinkedBlockingQueue<Object>(-1); return 0; } catch (IllegalArgumentException expected) {}
        LinkedBlockingQueue<Object> defaults = new LinkedBlockingQueue<Object>();
        check(defaults instanceof AbstractQueue && defaults instanceof Queue
            && defaults instanceof BlockingQueue && defaults instanceof Collection
            && defaults instanceof Iterable && defaults instanceof Serializable, "queue hierarchy");
        check(defaults.remainingCapacity() == Integer.MAX_VALUE && defaults.isEmpty(), "default capacity");
        check(defaults.peek() == null && defaults.poll() == null && !defaults.contains(null)
            && !defaults.remove(null) && !defaults.contains("absent"), "empty/null lookups");
        try { defaults.element(); return 0; } catch (NoSuchElementException expected) {}
        try { defaults.remove(); return 0; } catch (NoSuchElementException expected) {}
        Queue<Object> fifo = new LinkedBlockingQueue<Object>(2);
        check(fifo.add("first") && fifo.offer("first") && fifo.size() == 2, "ordered duplicates");
        check(!fifo.offer("full") && ((BlockingQueue<Object>) fifo).remainingCapacity() == 0, "full offer");
        try { fifo.add("full"); return 0; } catch (IllegalStateException expected) {}
        try { fifo.offer(null); return 0; } catch (NullPointerException expected) {}
        try { fifo.add(null); return 0; } catch (NullPointerException expected) {}
        check("first".equals(fifo.element()) && fifo.size() == 2, "element retains head");
        check("first".equals(fifo.poll()) && "first".equals(fifo.remove()) && fifo.isEmpty(), "FIFO removal");
        check(((BlockingQueue<Object>) fifo).remainingCapacity() == 2, "capacity restored");
        check(fifo.add(new Key(7)) && fifo.add(new Key(7)), "equal duplicates");
        equalityCalls = 0;
        check(fifo.contains(new Key(7)) && fifo.remove(new Key(7)) && fifo.size() == 1
            && equalityCalls == 2, "guest equality/first removal");
        try { fifo.contains(new ThrowingKey()); return 0; } catch (IllegalStateException expected) {
            check("queue equality failure".equals(expected.getMessage()), "equality fault preserved");
        }
        check(fifo.size() == 1 && ((Key) fifo.poll()).value == 7, "fault leaves values");
        Object self = new SelfFalse(); fifo.add(self); equalityCalls = 0;
        check(!fifo.contains(self) && !fifo.remove(self) && equalityCalls == 2 && fifo.poll() == self,
            "identity still invokes equals");
        fifo.add("a"); fifo.add("b"); fifo.clear();
        check(fifo.isEmpty() && ((BlockingQueue<Object>) fifo).remainingCapacity() == 2, "clear");
        RejectingQueue rejecting = new RejectingQueue();
        try { rejecting.add("rejected"); return 0; } catch (IllegalStateException expected) {}
        check(rejecting.offers == 1 && rejecting.isEmpty(), "add dispatches guest offer");
        OverridingQueue overriding = new OverridingQueue();
        check("guest peek".equals(overriding.element()) && "guest poll".equals(overriding.remove())
            && !overriding.isEmpty() && overriding.peeks == 1 && overriding.polls == 1
            && overriding.sizes == 1, "inherited methods dispatch guest overrides");
        check(defaults.equals(defaults) && !defaults.equals(new LinkedBlockingQueue<Object>()), "identity equality");
        return 1;
    }
    public static void retain() {
        retained = new LinkedBlockingQueue<Object>();
        retained.add(new StringBuilder().append("queue retains its element").toString());
    }
    public static String pollRetained() { return (String) retained.poll(); }
    public static void mutation() {
        mutating = new LinkedBlockingQueue<Object>(); mutating.add("before callback");
        mutating.contains(new MutatingKey());
    }
    public static void main(String[] args) {
        if (contract() != 1) throw new IllegalStateException("Queue contract failed");
        System.out.println("Immediate queue contract passed");
    }
}
