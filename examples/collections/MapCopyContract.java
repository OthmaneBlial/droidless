package org.droidless.collections;

import java.util.HashMap;
import java.util.LinkedHashMap;
import java.util.Map;

/** Native-map bulk copying; deliberately mutating input is a separate negative test. */
public class MapCopyContract {
    static Map<Object, String> source, target;
    static class Key {
        int id;
        Key(int id) { this.id=id; }
        public int hashCode() { return id; }
        public boolean equals(Object other) { System.gc(); return other instanceof Key && ((Key)other).id==id; }
    }
    static class ClearingKey {
        boolean clears;
        ClearingKey(boolean clears) { this.clears=clears; }
        public int hashCode() { return 1; }
        public boolean equals(Object other) {
            if (clears) { source.clear(); System.gc(); }
            return other instanceof ClearingKey;
        }
    }
    public static int run() {
        for (int kind=0; kind<2; kind++) {
            Map<Object,String> first = kind==0 ? new HashMap<Object,String>() : new LinkedHashMap<Object,String>();
            Map<Object,String> second = kind==0 ? new LinkedHashMap<Object,String>() : new HashMap<Object,String>();
            first.put(new Key(1), "old"); first.put("kept", "yes");
            second.put(new Key(1), new StringBuilder().append("replacement").toString());
            second.put(new Key(2), "new"); second.put(null,null);
            first.putAll(second);
            if (first.size()!=4 || !"replacement".equals(first.get(new Key(1))) || !"new".equals(first.get(new Key(2))) || !first.containsKey(null)) return 0;
            first.putAll(first);
            if (first.size()!=4 || !"yes".equals(first.get("kept"))) return 0;
            first.putAll(new HashMap<Object,String>());
            try { first.putAll(null); return 0; } catch (NullPointerException expected) {}
        }
        return 1;
    }
    public static Object prepareMutation() {
        source=new HashMap<Object,String>(); target=new HashMap<Object,String>();
        target.put(new ClearingKey(false), "old");
        Object key=new ClearingKey(true);
        source.put(key, new StringBuilder().append("snapshot stays alive").toString());
        return key;
    }
    public static void copyMutation() { target.putAll(source); }
    public static String afterMutation() { return target.get(new ClearingKey(false)); }
    public static void main(String[] args) {
        if (run()!=1) throw new IllegalStateException("Map copy contract failed");
        System.out.println("Map copy contract passed");
    }
}
