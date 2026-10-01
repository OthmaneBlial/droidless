package org.droidless.reflection;

/** Generic contract runnable on the desktop JDK and as compiled APK bytecode. */
public class ReflectionContract {
    static int initializations;
    static int constructors;
    public static class Thing {
        static { initializations++; }
        public int value;
        public Thing() { constructors++; value = 41; }
    }
    public static class ArrayComponent { static { initializations += 100; } }
    public static abstract class AbstractThing {}
    public interface Marker {}
    public static class NoDefault { public NoDefault(int n) {} }
    public static class PrivateConstructor {
        private PrivateConstructor() {}
        public static Object constructInside() throws Exception { return PrivateConstructor.class.newInstance(); }
    }
    public static class BrokenConstructor {
        public BrokenConstructor() { throw new IllegalArgumentException("original constructor exception"); }
    }
    public static class BrokenInitializer {
        static { if (initializations >= 0) throw new IllegalArgumentException("original initializer exception"); }
        public BrokenInitializer() {}
    }
    public static class Base {
        public int number = 11;
        public Object object = new Object();
        public long wide = 0x123456789abcdef0L;
        public static int shared = initializeShared();
        static int initializeShared() { initializations += 10; return 17; }
    }
    public static class Child extends Base { static { initializations += 1000; } }
    public static class Shadow extends Base { public int number = 23; }
    public interface Constants { Object VALUE = new Object(); }
    public interface SubConstants extends Constants {}
    public static int readAlias(Child object) { return object.number; }
    public static int run() throws Exception {
        Class<?> cls = Thing.class;
        if (initializations != 0 || !"org.droidless.reflection.ReflectionContract$Thing".equals(cls.getName())) return 0;
        ClassLoader loader = ReflectionContract.class.getClassLoader();
        if (cls.asSubclass(Object.class) != cls || cls.asSubclass(Thing.class) != cls || initializations != 0) return 0;
        try { cls.asSubclass(String.class); return 0; } catch (ClassCastException expected) {}
        try { cls.asSubclass(null); return 0; } catch (NullPointerException expected) {}
        if (loader.loadClass(cls.getName()) != cls || initializations != 0
            || loader.loadClass("java.lang.String") != String.class) return 0;
        System.gc();
        if (loader.loadClass(cls.getName()) != cls || initializations != 0) return 0;
        try { loader.loadClass("[I"); return 0; } catch (ClassNotFoundException expected) {}
        try { loader.loadClass("absent.Class"); return 0; } catch (ClassNotFoundException expected) {}
        try { loader.loadClass(null); return 0; } catch (NullPointerException expected) {}
        if (Class.forName(cls.getName()) != cls || initializations != 1) return 0;
        Thing first = (Thing) cls.newInstance();
        Thing second = (Thing) cls.newInstance();
        if (first == second || first.value != 41 || constructors != 2 || first.getClass() != cls) return 0;
        if (Class.forName(cls.getName()) != cls || initializations != 1) return 0;
        Class<?> array = Class.forName("[Lorg.droidless.reflection.ReflectionContract$ArrayComponent;");
        if (array != ArrayComponent[].class || initializations != 1) return 0;
        if (Class.forName("[[I") != int[][].class || !"[[I".equals(int[][].class.getName())) return 0;
        if (new Thing[1].getClass() != Thing[].class || new Object().getClass() != Object.class) return 0;
        if (Class.forName("java.lang.Object").newInstance().getClass() != Object.class) return 0;
        for (String name : new String[]{"absent.Class", "int", "", "java/lang/String", "[V", "[L[Ljava.lang.String;;", "[L;", "[Ljava.lang.String;;", "[Lmissing.Class;"}) {
            try { Class.forName(name); return 0; } catch (ClassNotFoundException expected) {}
        }
        try { Class.forName(null); return 0; } catch (NullPointerException expected) {}
        for (Class<?> type : new Class<?>[]{AbstractThing.class, Marker.class, NoDefault.class, Thing[].class}) {
            try { type.newInstance(); return 0; } catch (InstantiationException expected) {}
        }
        try { PrivateConstructor.class.newInstance(); return 0; } catch (IllegalAccessException expected) {}
        try { Class.forName("org.droidless.foreign.PackagePrivate").newInstance(); return 0; } catch (IllegalAccessException expected) {}
        if (PrivateConstructor.constructInside().getClass() != PrivateConstructor.class) return 0;
        try { BrokenConstructor.class.newInstance(); return 0; }
        catch (IllegalArgumentException expected) { if (!"original constructor exception".equals(expected.getMessage())) return 0; }
        try { Class.forName("org.droidless.reflection.ReflectionContract$BrokenInitializer"); return 0; }
        catch (ExceptionInInitializerError expected) { if (!"original initializer exception".equals(expected.getCause().getMessage())) return 0; }
        try { Class.forName("org.droidless.reflection.ReflectionContract$BrokenInitializer"); return 0; }
        catch (NoClassDefFoundError expected) {}
        if (Child.shared != 17 || initializations != 11) return 0; // Resolving an inherited static field initializes only its owner.
        Child child = new Child();
        Base base = child;
        if (initializations != 1011 || child.number != base.number || child.object != base.object || child.wide != base.wide) return 0;
        child.number = 29; child.wide = -1;
        if (base.number != 29 || base.wide != -1) return 0;
        Child.shared = 31; if (Base.shared != 31) return 0;
        Shadow shadow = new Shadow(); if (shadow.number != 23 || ((Base) shadow).number != 11) return 0;
        if (SubConstants.VALUE != Constants.VALUE) return 0;
        return 1;
    }
    public static void main(String[] args) throws Exception {
        if (run() != 1) throw new AssertionError("Reflection contract failed");
        System.out.println("Reflection passed");
    }
}
