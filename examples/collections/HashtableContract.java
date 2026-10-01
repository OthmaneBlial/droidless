package org.droidless.collections;

import java.util.Hashtable;
import java.util.HashMap;
import java.util.Map;
import java.util.Dictionary;
import java.io.Serializable;

/** Portable core Hashtable behavior, including synchronized guest equality. */
public final class HashtableContract {
    static Hashtable<Object, String> table;
    static int calls;
    static Key stored;
    static boolean failEquals;
    static class Key {
        final int id;
        Key(int id) { this.id = id; }
        public int hashCode() { return id; }
        public boolean equals(Object other) {
            if (!Thread.holdsLock(table)) throw new IllegalStateException("Hashtable equality outside monitor");
            if (this != stored) throw new IllegalStateException("Hashtable equality direction");
            calls++; System.gc();
            if (failEquals) throw new IllegalStateException("Hashtable equality failed");
            return other instanceof Key && ((Key)other).id == id;
        }
    }
    static void check(boolean value) { if (!value) throw new IllegalStateException("Hashtable contract failed"); }
    public static int run() {
        table = new Hashtable<Object, String>();
        check(table instanceof Dictionary && table instanceof Map && table instanceof Cloneable && table instanceof Serializable);
        check(table.isEmpty() && table.get("absent") == null);
        stored = new Key(7);
        check(table.put(stored, "old") == null);
        check("old".equals(table.put(new Key(7), "new")) && table.size() == 1);
        check("new".equals(table.get(new Key(7))) && table.containsKey(new Key(7)) && table.containsValue("new"));
        try { table.put(null, "invalid"); return 0; } catch (NullPointerException expected) {}
        try { table.put("invalid", null); return 0; } catch (NullPointerException expected) {}
        try { table.get(null); return 0; } catch (NullPointerException expected) {}
        try { table.remove(null); return 0; } catch (NullPointerException expected) {}
        try { table.containsKey(null); return 0; } catch (NullPointerException expected) {}
        try { table.containsValue(null); return 0; } catch (NullPointerException expected) {}
        check(table.size() == 1 && !Thread.holdsLock(table));
        synchronized (table) {
            check("new".equals(table.get(new Key(7))) && Thread.holdsLock(table));
        }
        failEquals = true;
        try { table.get(new Key(7)); return 0; } catch (IllegalStateException expected) {
            check("Hashtable equality failed".equals(expected.getMessage()));
        } finally { failEquals = false; }
        check(!Thread.holdsLock(table) && "new".equals(table.get(new Key(7))));
        Map<Object, String> copied = new HashMap<Object, String>();
        copied.putAll(table); check(copied.size() == 1 && copied.containsValue("new"));
        Map<Object, String> source = new HashMap<Object, String>(); source.put("other", "kept");
        table.putAll(source); System.gc();
        check(table.size() == 2 && "kept".equals(table.get("other")));
        check("new".equals(table.remove(new Key(7))) && table.remove("absent") == null);
        table.clear(); check(table.isEmpty() && calls > 0 && !Thread.holdsLock(table));
        table = null; stored = null; System.gc();
        return 1;
    }
    public static void main(String[] args) {
        if (run() != 1) throw new IllegalStateException("Hashtable contract failed");
        System.out.println("Hashtable contract passed");
    }
}
