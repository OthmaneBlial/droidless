package org.droidless.collections;
import java.lang.ref.Reference;
import java.lang.ref.WeakReference;

public final class WeakReferenceContract {
    static final class Sub extends WeakReference<Object> {
        Object anchor;
        Sub(Object object) { super(object); anchor=new Object(); }
    }
    static Object strong;
    static Sub weak;
    static WeakReference<Object> cycle;
    static WeakReference<Object> empty;
    public static Object prepare() {
        strong=new Object(); weak=new Sub(strong);
        Object[] loop=new Object[1]; loop[0]=loop; cycle=new WeakReference<Object>(loop);
        empty=new WeakReference<Object>(null,null);
        System.gc(); check(weak.get()==strong && cycle.get()==loop && empty.get()==null);
        return strong;
    }
    public static Object anchored() { return weak.anchor; }
    public static int retained() {
        Reference<Object> erased=weak;
        check(erased.get()==strong && strong!=null && cycle.get()==null && empty.get()==null);
        return 1;
    }
    public static void release() { strong=null; }
    public static int collected() { check(weak.get()==null && weak.anchor!=null); return 1; }
    public static Object reset() { strong=new Object(); weak=new Sub(strong); return strong; }
    public static void clear() { Reference<Object> erased=weak; erased.clear(); }
    public static int cleared() { check(weak.get()==null && strong!=null); return 1; }
    public static void drop() { strong=null; weak=null; cycle=null; empty=null; }
    static void check(boolean okay) { if (!okay) throw new IllegalStateException("weak reference contract failed"); }
}
