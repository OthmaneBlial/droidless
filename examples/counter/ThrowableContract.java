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
    static class CauseTrace extends RuntimeException {
        String observedMessage;
        Throwable observedCause;
        CauseTrace(String message, Throwable cause) { super(message, cause); }
        @Override public Throwable fillInStackTrace() {
            System.gc(); observedMessage = getMessage(); observedCause = getCause();
            return super.fillInStackTrace();
        }
    }
    static class CauseTraceFault extends RuntimeException {
        CauseTraceFault(String message, Throwable cause) { super(message, cause); }
        @Override public Throwable fillInStackTrace() {
            System.gc(); throw new IllegalStateException("cause trace failed");
        }
    }
    public static int verifyCauseConstructors() {
        Throwable cause = new IllegalArgumentException("inner cause");
        String message = new StringBuilder().append("outer cause").toString();
        Throwable[] wrappers = {new Throwable(message, cause), new Exception(message, cause),
            new RuntimeException(message, cause), new IllegalStateException(message, cause),
            new CauseTrace(message, cause)};
        System.gc();
        for (Throwable wrapper : wrappers)
            if (wrapper.getMessage() != message || wrapper.getCause() != cause)
                throw new AssertionError("constructor cause/message identity");
        Throwable empty = new Throwable(null, null);
        if (empty.getMessage() != null || empty.getCause() != null)
            throw new AssertionError("null message/cause");
        saved = wrappers[4];
        return 1;
    }
    // API-21 libcore assigns message/cause before virtual fillInStackTrace;
    // desktop Java 17 assigns them afterward. This check is Android-profile-only.
    public static int api21CauseConstructorState() {
        CauseTrace wrapper = (CauseTrace) saved;
        return wrapper.observedMessage == wrapper.getMessage()
            && wrapper.observedCause == wrapper.getCause() ? 1 : 0;
    }
    public static void causeConstructorFault() {
        new CauseTraceFault("fault wrapper", new IllegalArgumentException("fault cause"));
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
    public static int logSaved(int level) {
        String tag = new StringBuilder().append("ThrowableContract").toString();
        String message = new StringBuilder().append("logged original exception").toString();
        switch (level) {
            case 0: return android.util.Log.d(tag, message, saved);
            case 1: return android.util.Log.i(tag, message, saved);
            case 2: return android.util.Log.w(tag, message, saved);
            default: return android.util.Log.e(tag, message, saved);
        }
    }
    public static int logNulls() {
        android.util.Log.d(null, null, null);
        android.util.Log.i(null, null, null);
        android.util.Log.w(null, null, null);
        android.util.Log.e(null, null, null);
        return 1;
    }
    public static void main(String[] args) {
        verifyCauseConstructors();
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
