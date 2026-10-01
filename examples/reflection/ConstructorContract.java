package org.droidless.reflection;

import java.lang.reflect.Constructor;
import java.lang.reflect.InvocationTargetException;
import java.lang.annotation.Inherited;
import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;

/** Portable reference-argument construction; no desktop host classes are loaded by DROIDLESS. */
public class ConstructorContract {
    static int initialized;
    static IllegalArgumentException original;
    public static class Thing {
        static { initialized++; }
        final String text;
        final Object identity;
        public Thing(String text,Object identity) {
            System.gc();
            if ("fail".equals(text)) { original=new IllegalArgumentException("constructor cause");throw original; }
            this.text=text;this.identity=identity;
        }
    }
    public static class PrivateThing { private PrivateThing() { System.gc(); } }
    public static abstract class AbstractThing { public AbstractThing() {} }
    public static class PrimitiveThing { public PrimitiveThing(int value) {} }
    @Inherited @Retention(RetentionPolicy.RUNTIME) public @interface Mark { Class<?> value(); }
    @Retention(RetentionPolicy.RUNTIME) public @interface Direct { String value(); }
    @Retention(RetentionPolicy.CLASS) public @interface Hidden {}
    @Mark(Thing.class) @Direct("parent") @Hidden public static class Annotated {}
    public static class AnnotationChild extends Annotated {}
    @Mark(PrivateThing.class) public static class AnnotationOverride extends AnnotationChild {}
    static void require(boolean value) { if (!value) throw new AssertionError("constructor contract"); }
    public static int contract() throws Exception {
        require(Annotated.class.getAnnotation(Mark.class).value()==Thing.class
            && AnnotationChild.class.getAnnotation(Mark.class).value()==Thing.class
            && AnnotationOverride.class.getAnnotation(Mark.class).value()==PrivateThing.class
            && "parent".equals(Annotated.class.getAnnotation(Direct.class).value())
            && AnnotationChild.class.getAnnotation(Direct.class)==null
            && Annotated.class.getAnnotation(Hidden.class)==null);
        require(initialized==0);
        ClassLoader loader=ConstructorContract.class.getClassLoader();
        require(loader==Thing.class.getClassLoader());
        Class<?> loaded=Class.forName("org.droidless.reflection.ConstructorContract$Thing",false,loader);
        require(loaded==Thing.class && initialized==0);
        try { Class.forName(Thing.class.getName(),false,null);throw new AssertionError("bootstrap loaded APK class"); }
        catch (ClassNotFoundException expected) {}
        require(Class.forName("java.lang.String",false,null)==String.class);
        Constructor<?> constructor=loaded.getConstructor(String.class,Object.class);
        require(initialized==0 && !constructor.isAccessible());
        Object identity=new Object();
        Thing first=(Thing)constructor.newInstance("hello",identity);
        Thing second=(Thing)constructor.newInstance(null,null);
        require(initialized==1 && first.identity==identity && "hello".equals(first.text) && second.text==null && first!=second);
        int faults=0;
        try { Thing.class.getConstructor(Object.class); } catch (NoSuchMethodException expected) {faults++;}
        try { constructor.newInstance("short"); } catch (IllegalArgumentException expected) {faults++;}
        try { constructor.newInstance(identity,identity); } catch (IllegalArgumentException expected) {faults++;}
        try { constructor.newInstance("fail",identity); }
        catch (InvocationTargetException expected) {require(expected.getCause()==original);faults++;}
        try { PrivateThing.class.getConstructor(); } catch (NoSuchMethodException expected) {faults++;}
        Constructor<PrivateThing> hidden=PrivateThing.class.getDeclaredConstructor();
        try { hidden.newInstance(); } catch (IllegalAccessException expected) {faults++;}
        hidden.setAccessible(true);require(hidden.isAccessible() && hidden.newInstance().getClass()==PrivateThing.class);
        hidden.setAccessible(false);
        try { hidden.newInstance(); } catch (IllegalAccessException expected) {faults++;}
        try { AbstractThing.class.getDeclaredConstructor().newInstance(); } catch (InstantiationException expected) {faults++;}
        try { Class.forName("org.droidless.foreign.PackagePrivate").getConstructor().newInstance(); } catch (IllegalAccessException expected) {faults++;}
        require(faults==9);
        String text="a😀.a";
        require(text.indexOf('.')==3 && text.indexOf('.',4)==-1 && text.indexOf(0x1f600)==1 && text.indexOf(0xd83d)==1
            && text.indexOf("a",1)==4 && text.indexOf("",99)==5 && text.indexOf('.',-8)==3 && text.indexOf(0x110000)==-1
            && text.lastIndexOf('.')==3 && text.lastIndexOf("",-1)==-1);
        System.gc();return 1;
    }
    public static void unsupportedPrimitive() throws Exception { PrimitiveThing.class.getConstructor(int.class).newInstance(1); }
    public static void main(String[] args) throws Exception { require(contract()==1);System.out.println("Constructor contract passed"); }
}
