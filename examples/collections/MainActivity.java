package org.droidless.collections;

import android.app.Activity;
import android.os.Bundle;
import android.widget.TextView;
import java.util.HashSet;
import java.util.HashMap;
import java.util.Set;
import java.util.Map;
import java.util.Iterator;
import java.util.Collections;
import java.util.NoSuchElementException;
import java.util.ConcurrentModificationException;

/** Authored generic API conformance; not independent notes-app compatibility. */
public class MainActivity extends Activity {
    static Iterator<Object> retained;
    static Map<Object,String> retainedMap;
    static int equalityCalls;
    static Set<Object> mutatingSet;
    static Map<Object,String> mutatingMap;
    public void onCreate(Bundle state) {
        super.onCreate(state);
        TextView label = new TextView(this);
        label.setText(contract() == 1 ? "Collections passed" : "BROKEN collections");
        setContentView(label);
    }
    static class Key {
        int value;
        Key(int value) { this.value = value; }
        public boolean equals(Object other) { equalityCalls++; return other instanceof Key && ((Key) other).value == value; }
        public int hashCode() { return value; }
    }
    static class MutatingKey {
        public boolean equals(Object other) {
            if (mutatingSet != null) mutatingSet.add("mutation");
            if (mutatingMap != null) mutatingMap.put("mutation", "nested value");
            return false;
        }
    }
    public static int contract() {
        if (ListContract.contract() != 1 || ListContract.mutationContract() != 1
            || QueueContract.contract() != 1 || MapCopyContract.run() != 1) return 0;
        if (MainActivity.class != MainActivity.class || MainActivity.class == (Object) Key.class) return 0;
        if (!"org.droidless.collections".equals(MainActivity.class.getPackage().getName()) || String[].class.getPackage() != null) return 0;
        Map<Class<?>,String> classes = new HashMap<Class<?>,String>();
        classes.put(MainActivity.class, "same class key");
        if (!"same class key".equals(classes.get(MainActivity.class))) return 0;
        try { new HashSet<Object>(-1); return 0; } catch (IllegalArgumentException expected) {}
        try { new HashMap<Object,Object>(-1); return 0; } catch (IllegalArgumentException expected) {}
        Set<Object> set = new HashSet<Object>();
        if (!set.isEmpty() || !set.add(new StringBuilder().append("value").toString())) return 0;
        if (set.add(new StringBuilder().append("value").toString()) || !set.contains("value")) return 0;
        if (!set.add(null) || set.add(null) || !set.contains(null)) return 0;
        if (!set.add(new Key(7)) || set.add(new Key(7)) || !set.contains(new Key(7)) || set.size() != 3) return 0;
        Iterator<Object> iterator = set.iterator();
        try { iterator.remove(); return 0; } catch (IllegalStateException expected) {}
        Object first = iterator.next(); iterator.remove();
        if (set.contains(first) || set.size() != 2) return 0;
        try { iterator.remove(); return 0; } catch (IllegalStateException expected) {}
        while (iterator.hasNext()) iterator.next();
        try { iterator.next(); return 0; } catch (NoSuchElementException expected) {}
        iterator = set.iterator(); set.add("change");
        try { iterator.next(); return 0; } catch (ConcurrentModificationException expected) {}
        iterator = set.iterator(); if (set.add("change")) return 0; iterator.next();
        set.clear(); if (!set.isEmpty() || set.remove("absent")) return 0;
        Set<Object> view = Collections.unmodifiableSet(set);
        if (view == set || view instanceof HashSet || !view.isEmpty()) return 0;
        set.add("live"); if (!view.contains("live") || view.size() != 1) return 0;
        try { view.add("forbidden"); return 0; } catch (UnsupportedOperationException expected) {}
        try { view.clear(); return 0; } catch (UnsupportedOperationException expected) {}
        iterator = view.iterator();
        try { iterator.remove(); return 0; } catch (UnsupportedOperationException expected) {}
        iterator.next();
        try { iterator.remove(); return 0; } catch (UnsupportedOperationException expected) {}
        Map<Object,String> map = new HashMap<Object,String>(4);
        if (!map.isEmpty() || map.put(new Key(9), "first") != null) return 0;
        if (!"first".equals(map.put(new Key(9), "second")) || map.size() != 1) return 0;
        if (!"second".equals(map.get(new Key(9))) || !map.containsKey(new Key(9)) || !map.containsValue("second")) return 0;
        if (map.put(null, null) != null || !map.containsKey(null) || !map.containsValue(null) || map.get(null) != null) return 0;
        if (map.remove(null) != null || map.containsKey(null) || map.size() != 1) return 0;
        if (!"second".equals(map.remove(new Key(9))) || !map.isEmpty()) return 0;
        map.put("clear", "discard"); map.clear();
        set.add("seed"); mutatingSet = set;
        try { set.contains(new MutatingKey()); return 0; } catch (ConcurrentModificationException expected) {}
        mutatingSet = null;
        map.put("seed", "seed"); mutatingMap = map;
        try { map.get(new MutatingKey()); return 0; } catch (ConcurrentModificationException expected) {}
        mutatingMap = null; map.clear();
        return map.isEmpty() && equalityCalls > 0 ? 1 : 0;
    }
    public static void retain() {
        Set<Object> set = new HashSet<Object>();
        set.add(new StringBuilder().append("iterator keeps its owner").toString());
        retained = Collections.unmodifiableSet(set).iterator();
        retainedMap = new HashMap<Object,String>();
        retainedMap.put(new Key(42), new StringBuilder().append("map keeps key and value").toString());
    }
    public static String nextRetained() { return (String) retained.next(); }
    public static String retainedValue() { return retainedMap.get(new Key(42)); }
}
