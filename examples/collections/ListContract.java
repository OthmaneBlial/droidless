package org.droidless.collections;

import java.io.Serializable;
import java.util.AbstractList;
import java.util.ArrayList;
import java.util.ConcurrentModificationException;
import java.util.Iterator;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.NoSuchElementException;
import java.util.RandomAccess;

/** The same generic contract runs as desktop Java and as compiled guest DEX. */
public class ListContract {
    static List<Object> mutating;
    static List<Object> replacing;
    static Iterator<Object> retained;
    static class Key {
        final int value;
        Key(int value) { this.value = value; }
        public boolean equals(Object other) {
            return other instanceof Key && ((Key) other).value == value;
        }
        public int hashCode() { return value; }
    }
    static class MutatingKey {
        public boolean equals(Object other) { mutating.add("nested"); return false; }
    }
    static class ReplacingKey {
        public boolean equals(Object other) { replacing.set(1, "target"); return "target".equals(other); }
    }
    static class SelfKey {
        static int calls;
        public boolean equals(Object other) { calls++; return this == other; }
    }
    static class ChildList extends ArrayList<Object> {
        static int removals;
        public Object remove(int index) { removals++; return super.remove(index); }
    }
    static class EvictingMap extends LinkedHashMap<Object,Object> {
        protected boolean removeEldestEntry(Map.Entry<Object,Object> eldest) { return true; }
    }
    static void check(boolean condition, String message) {
        if (!condition) throw new IllegalStateException(message);
    }
    public static int contract() {
        try { new ArrayList<Object>(-1); return 0; } catch (IllegalArgumentException expected) {}
        List<Object> list = new ChildList();
        check(list instanceof AbstractList && list instanceof RandomAccess
            && list instanceof Cloneable && list instanceof Serializable, "list hierarchy");
        check(list.isEmpty() && list.size() == 0, "empty list");
        check(list.add("a") && list.add(null) && list.add("a") && list.add(null), "duplicate add");
        check(list.size() == 4 && list.indexOf(null) == 1 && list.lastIndexOf(null) == 3, "null indices");
        check(list.indexOf("missing") == -1 && list.lastIndexOf("missing") == -1, "absent indices");
        check(list.contains(new StringBuilder().append("a").toString()), "string equality");
        list.add(0, "start"); list.add(list.size(), "end"); list.add(2, new Key(7));
        check(list.size() == 7 && list.indexOf(new Key(7)) == 2, "indexed insertion/equality");
        check(list.remove(null) && list.indexOf(null) == 4 && list.size() == 6, "first null removal");
        check("end".equals(list.remove(list.size() - 1)), "indexed removal return");
        Iterator<Object> iterator = list.iterator();
        ChildList.removals = 0;
        try { iterator.remove(); return 0; } catch (IllegalStateException expected) {}
        check("start".equals(list.set(0, "changed")), "set return");
        check("changed".equals(iterator.next()), "set preserves iterator");
        iterator.remove();
        check(ChildList.removals == 1, "iterator uses guest remove override");
        check("a".equals(iterator.next()) && list.size() == 4, "iterator removal cursor");
        try { iterator.remove(); iterator.remove(); return 0; } catch (IllegalStateException expected) {}
        check(ChildList.removals == 2, "second guest remove override");
        check(iterator.next() instanceof Key && "a".equals(iterator.next()) && iterator.next() == null, "iteration order");
        check(!iterator.hasNext(), "exhaustion state");
        try { iterator.next(); return 0; } catch (NoSuchElementException expected) {}
        iterator = list.iterator(); list.add("change");
        try { iterator.next(); return 0; } catch (ConcurrentModificationException expected) {}
        iterator = list.iterator(); iterator.next(); list.remove(0);
        try { iterator.remove(); return 0; } catch (ConcurrentModificationException expected) {}
        list.clear(); check(list.isEmpty() && !list.remove("absent"), "clear");
        list.add("one");
        int[] bad = {-1, 1, Integer.MIN_VALUE, Integer.MAX_VALUE};
        for (int index : bad) {
            try { list.get(index); return 0; } catch (IndexOutOfBoundsException expected) {}
            try { list.set(index, "bad"); return 0; } catch (IndexOutOfBoundsException expected) {}
            try { list.remove(index); return 0; } catch (IndexOutOfBoundsException expected) {}
            try { list.add(index == 1 ? 2 : index, "bad"); return 0; } catch (IndexOutOfBoundsException expected) {}
        }
        check(list.size() == 1 && "one".equals(list.get(0)), "bounds keep old data");
        list.add(new Key(8)); list.add(new Key(8));
        check(list.indexOf(new Key(8)) == 1 && list.lastIndexOf(new Key(8)) == 2
            && list.remove(new Key(8)) && list.size() == 2, "guest equality and duplicates");
        replacing = new ArrayList<Object>(); replacing.add("first"); replacing.add("old");
        check(replacing.indexOf(new ReplacingKey()) == 1, "search observes nonstructural replacement");
        replacing = null;
        SelfKey.calls = 0; SelfKey self = new SelfKey(); list.clear(); list.add(self);
        check(list.indexOf(self) == 0 && SelfKey.calls == 1, "guest equals for identical references");
        Map<Object,String> map = new LinkedHashMap<Object,String>(2);
        check(map.put(new Key(9), "first") == null
            && "first".equals(map.put(new Key(9), "second")) && map.size() == 1, "inherited linked map");
        map.put(null, null);
        check(map.containsKey(null) && map.containsValue(null)
            && "second".equals(map.remove(new Key(9))), "linked map null/removal");
        map.clear(); check(map.isEmpty(), "linked map clear");
        return 1;
    }
    public static int mutationContract() {
        mutating = new ArrayList<Object>(); mutating.add("seed");
        try { mutating.contains(new MutatingKey()); return 0; }
        catch (ConcurrentModificationException expected) { return mutating.size() == 2 ? 1 : 0; }
        finally { mutating = null; }
    }
    public static void retain() {
        List<Object> list = new ArrayList<Object>();
        list.add(new StringBuilder().append("list iterator keeps its owner").toString());
        retained = list.iterator();
    }
    public static String nextRetained() { return (String) retained.next(); }
    public static void main(String[] args) {
        check(contract() == 1, "list contract failed");
        System.out.println("List contract passed");
    }
}
