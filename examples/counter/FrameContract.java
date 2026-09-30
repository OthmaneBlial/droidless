package org.droidless.counter;

/** Ordinary Java recursion/exception contract; no worker execution claim. */
public class FrameContract {
    static int finallyRuns;
    public static String nested(int depth, String value) {
        if (depth == 0) { System.gc(); return value + "/leaf"; }
        return nested(depth - 1, value) + "!";
    }
    public static long wide(int depth, long value) {
        if (depth == 0) return value;
        return wide(depth - 1, value + 1) + depth;
    }
    public static void throwNested(int depth) {
        if (depth == 0) throw new IllegalStateException("stack failure");
        throwNested(depth - 1);
    }
    public static String caught() {
        try {
            try { throwNested(4); return "unreachable"; }
            finally { finallyRuns++; }
        } catch (IllegalArgumentException wrong) { return "wrong catch"; }
        catch (IllegalStateException expected) { return expected.getMessage(); }
    }
    public static void unsupported(int depth) {
        if (depth == 0) { System.exit(3); return; }
        unsupported(depth - 1);
    }
    public static int contract() {
        finallyRuns = 0;
        return "slice/leaf!!!".equals(nested(3, "slice")) && wide(4,100)==114
            && "stack failure".equals(caught()) && finallyRuns==1 ? 1 : 0;
    }
    public static void main(String[] args) {
        if (contract()!=1) throw new IllegalStateException("Frame contract failed");
        System.out.println("Frame contract passed");
    }
}
