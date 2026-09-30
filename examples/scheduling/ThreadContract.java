package org.droidless.scheduling;

/** Unstarted-thread metadata/manual run conformance; no background-thread claim. */
public class ThreadContract {
    static int calls;
    public static int contract() {
        Thread main = Thread.currentThread();
        if (main != Thread.currentThread() || !main.isAlive() || main.getId() <= 0) return 0;
        Thread first = new Thread("unstarted");
        Thread second = new Thread(new Runnable() { public void run() { calls++; } }, "manual");
        if (first.isAlive() || first.getId() == second.getId() || first.getId() == main.getId()) return 0;
        first.setName("renamed"); if (!"renamed".equals(first.getName())) return 0;
        try { first.setName(null); return 0; } catch (NullPointerException expected) {}
        try { new Thread((String) null); return 0; } catch (NullPointerException expected) {}
        calls = 0; second.run(); first.run();
        return calls == 1 && !second.isAlive() && Thread.currentThread() == main ? 1 : 0;
    }
    public static void main(String[] args) {
        if (contract() != 1) throw new IllegalStateException("Thread metadata contract failed");
        System.out.println("Thread metadata contract passed");
    }
}
