package org.droidless.collections;

import java.io.Serializable;
import java.util.Collections;
import java.util.Iterator;
import java.util.List;
import java.util.NoSuchElementException;
import java.util.RandomAccess;
import java.util.concurrent.CopyOnWriteArrayList;

/** Same snapshot contract runs as compiled guest DEX and desktop Java. */
public class SnapshotContract {
    static CopyOnWriteArrayList<Object> changing, shared;
    static Iterator<Object> retained;
    static Writer writer;
    static class Key {
        int value;
        static int calls;
        Key(int value) { this.value=value; }
        public boolean equals(Object other) { calls++; System.gc(); return other instanceof Key && ((Key)other).value==value; }
        public int hashCode() { return value; }
    }
    static class ClearingKey {
        public boolean equals(Object other) { changing.clear(); System.gc(); return "target".equals(other); }
    }
    static class Writer extends Thread {
        Writer() { super("snapshot writer"); }
        public void run() { shared.set(0,"worker changed"); shared.add(null); }
    }
    static void check(boolean condition) { if (!condition) throw new IllegalStateException("Snapshot contract failed"); }
    public static int run() {
        CopyOnWriteArrayList<Object> list = new CopyOnWriteArrayList<Object>();
        check(list instanceof List && list instanceof RandomAccess && list instanceof Cloneable && list instanceof Serializable);
        check(list.isEmpty() && list.add("first") && list.add(null) && list.add("first"));
        check(list.size()==3 && list.indexOf(null)==1 && list.lastIndexOf("first")==2);
        Iterator<Object> snapshot = list.iterator();
        try { snapshot.remove(); return 0; } catch (UnsupportedOperationException expected) {}
        check("first".equals(list.set(0,"changed")));
        list.add(1,"inserted"); check(list.remove(null));
        check("first".equals(list.remove(2))); list.clear(); list.add("new");
        System.gc();
        check("first".equals(snapshot.next()) && snapshot.next()==null && "first".equals(snapshot.next()));
        check(!snapshot.hasNext());
        try { snapshot.next(); return 0; } catch (NoSuchElementException expected) {}
        try { snapshot.remove(); return 0; } catch (UnsupportedOperationException expected) {}
        list.clear(); list.add("a"); list.add("b"); list.add("c");
        snapshot=list.iterator();
        while (snapshot.hasNext()) check(list.remove(snapshot.next()));
        check(list.isEmpty());
        Key self=new Key(7); list.add(self); Key.calls=0;
        check(list.contains(self) && Key.calls==1 && list.indexOf(new Key(7))==0);
        list.add(new Key(7)); check(list.remove(new Key(7)) && list.size()==1);
        int[] bad={-1,1,Integer.MIN_VALUE,Integer.MAX_VALUE};
        for (int index:bad) {
            try { list.get(index); return 0; } catch (IndexOutOfBoundsException expected) {}
            try { list.set(index,null); return 0; } catch (IndexOutOfBoundsException expected) {}
            try { list.remove(index); return 0; } catch (IndexOutOfBoundsException expected) {}
            try { list.add(index==1 ? 2 : index,null); return 0; } catch (IndexOutOfBoundsException expected) {}
        }
        List<Object> view=Collections.unmodifiableList(list); snapshot=view.iterator(); list.clear();
        check(snapshot.next() instanceof Key && !snapshot.hasNext());
        try { snapshot.remove(); return 0; } catch (UnsupportedOperationException expected) {}
        changing=new CopyOnWriteArrayList<Object>();
        changing.add(new StringBuilder().append("first").toString());
        changing.add(new StringBuilder().append("target").toString());
        check(changing.indexOf(new ClearingKey())==1 && changing.isEmpty());
        changing=null;
        return 1;
    }
    public static Object retain() {
        CopyOnWriteArrayList<Object> list=new CopyOnWriteArrayList<Object>();
        list.add(new StringBuilder().append("snapshot keeps old value").toString());
        retained=Collections.unmodifiableList(list).iterator();
        list.set(0,"replacement"); list.clear();
        return list;
    }
    public static String nextRetained() { return (String)retained.next(); }
    public static void release() { retained=null; }
    public static Object prepareWriteMutation() {
        changing=new CopyOnWriteArrayList<Object>();
        Object value=new StringBuilder().append("target").toString(); changing.add(value); return value;
    }
    public static void removeMutation() { changing.remove(new ClearingKey()); }
    public static int mutationSize() { return changing.size(); }
    public static void prepareWorker() {
        shared=new CopyOnWriteArrayList<Object>(); shared.add("worker original");
        retained=shared.iterator(); writer=new Writer(); writer.start();
    }
    public static int verifyWorker() {
        return !writer.isAlive() && shared.size()==2 && "worker changed".equals(shared.get(0))
            && "worker original".equals(retained.next()) && !retained.hasNext() ? 1 : 0;
    }
    public static void main(String[] args) throws Exception {
        check(run()==1); prepareWorker(); writer.join(); check(verifyWorker()==1);
        System.out.println("Snapshot contract passed");
    }
}
