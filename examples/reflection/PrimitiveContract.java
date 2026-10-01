package org.droidless.reflection;

import java.util.HashMap;

/** Primitive Class identity/field contract, shared with desktop Java. */
public class PrimitiveContract {
    public static Class<?> readInt() { return Integer.TYPE; }
    public static int run() throws Exception {
        Class<?>[] types = {Void.TYPE, Boolean.TYPE, Byte.TYPE, Character.TYPE,
            Short.TYPE, Integer.TYPE, Long.TYPE, Float.TYPE, Double.TYPE};
        Class<?>[] wrappers = {Void.class, Boolean.class, Byte.class, Character.class,
            Short.class, Integer.class, Long.class, Float.class, Double.class};
        String[] names = {"void","boolean","byte","char","short","int","long","float","double"};
        HashMap<Class<?>, String> metadata = new HashMap<Class<?>, String>();
        for (int i=0; i<types.length; i++) {
            if (types[i] == wrappers[i] || !names[i].equals(types[i].getName())
                    || !names[i].equals(types[i].toString())
                    || !("class " + wrappers[i].getName()).equals(wrappers[i].toString())) return 0;
            if (Class.forName(wrappers[i].getName()) != wrappers[i]) return 0;
            try { Class.forName(names[i]); return 0; } catch (ClassNotFoundException expected) {}
            try { types[i].newInstance(); return 0; } catch (InstantiationException expected) {}
            if (i==0) {
                try { wrappers[i].newInstance(); return 0; } catch (IllegalAccessException expected) {}
            } else {
                try { wrappers[i].newInstance(); return 0; } catch (InstantiationException expected) {}
            }
            metadata.put(types[i], names[i]);
        }
        System.gc();
        if (metadata.size()!=9 || !"int".equals(metadata.get(readInt()))) return 0;
        if (readInt()!=int.class || Double.TYPE!=double.class || Void.TYPE!=void.class) return 0;
        if (Class.forName("[I")!=int[].class || !"[I".equals(int[].class.getName())) return 0;
        if (!"class [I".equals(int[].class.toString()) || !"class [[Ljava.lang.String;".equals(String[][].class.toString())
                || !"interface java.lang.Runnable".equals(Runnable.class.toString())
                || !"interface java.util.Map".equals(java.util.Map.class.toString())
                || !"class java.lang.String".equals(String.class.toString())
                || !"interface org.droidless.reflection.ReflectionContract$Constants".equals(ReflectionContract.Constants.class.toString())
                || !"class org.droidless.reflection.PrimitiveContract".equals(PrimitiveContract.class.toString())) return 0;
        return 1;
    }
    public static void main(String[] args) throws Exception {
        if (run()!=1) throw new IllegalStateException("Primitive contract failed");
        System.out.println("Primitive contract passed");
    }
}
