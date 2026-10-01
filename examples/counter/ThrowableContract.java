package org.droidless.counter;

/** Retained exception diagnostics; stack formatting is VM-specific. */
public class ThrowableContract {
    static Throwable saved;
    static class Localized extends RuntimeException {
        Localized() { super("raw message"); }
        @Override public String getLocalizedMessage() { System.gc(); return "localized message"; }
    }
    static class BadDescription extends RuntimeException {
        @Override public String toString() { throw new IllegalStateException("description failed"); }
    }
    static class NoTrace extends RuntimeException {
        @Override public Throwable fillInStackTrace() { System.gc(); return this; }
    }
    static class Chain extends RuntimeException {
        Throwable next;
        Chain(String message) { super(message); }
        @Override public Throwable getCause() { System.gc(); return next; }
    }
    static Throwable leaf(int mode) {
        if (mode == 1) {
            int zero = 0;
            try { int unused = 8 / zero; }
            catch (ArithmeticException expected) { return expected; }
        }
        if (mode == 2) return new Localized();
        if (mode == 3) return new BadDescription();
        if (mode == 5) return new NoTrace();
        if (mode == 4) {
            Chain a = new Chain("outer");
            Chain b = new Chain("inner");
            a.next = b; b.next = a;
            return a;
        }
        return new IllegalStateException("saved message");
    }
    static Throwable outer(int mode) { return leaf(mode); }
    public static Throwable capture(int mode) {
        saved = outer(mode);
        System.gc();
        return saved;
    }
    public static int printSaved() {
        System.gc();
        saved.printStackTrace();
        return 1;
    }
    public static Throwable refresh() { return saved.fillInStackTrace(); }
    public static void main(String[] args) {
        for (int mode : new int[]{0, 1, 2, 4, 5}) {
            capture(mode);
            if (printSaved() != 1) throw new IllegalStateException("printing did not return");
        }
        capture(3);
        try { printSaved(); throw new AssertionError("description fault swallowed"); }
        catch (IllegalStateException expected) {
            if (!"description failed".equals(expected.getMessage())) throw expected;
        }
        System.out.println("Throwable contract passed");
    }
}
