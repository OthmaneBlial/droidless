package org.droidless.images;

import android.content.Context;
import android.content.ContextWrapper;
import android.content.res.Resources;
import android.content.res.TypedArray;
import android.view.ContextThemeWrapper;
import android.view.LayoutInflater;
import android.view.View;
import android.util.AttributeSet;

/** Real theme/service callbacks, independent snapshots, inflater factories and faults. */
public final class ThemedContextContract {
    static class Base extends StyleColorContract.Context {
        int resources, services;
        boolean resourceFault, serviceFault;
        final Object service = new Object();
        public Resources getResources() {
            resources++; System.gc();
            if (resourceFault) throw new IllegalStateException("resources failed");
            return super.getResources();
        }
        public Object getSystemService(String name) {
            services++; System.gc();
            if (serviceFault) throw new IllegalStateException("service failed");
            return name.equals(Context.LAYOUT_INFLATER_SERVICE) ? super.getSystemService(name) : service;
        }
    }
    static class Themed extends ContextThemeWrapper {
        int applies;
        boolean first, fail;
        Themed() { super(); }
        Themed(Context base, int style) { super(base, style); }
        void attach(Context base) { super.attachBaseContext(base); }
        protected void onApplyThemeResource(Resources.Theme theme, int style, boolean first) {
            applies++; this.first=first; System.gc();
            if (fail) throw new IllegalStateException("apply failed");
            super.onApplyThemeResource(theme,style,first);
        }
    }
    static class Factory implements LayoutInflater.Factory {
        Context seen;
        public View onCreateView(String name, Context context, AttributeSet attrs) {
            seen=context; System.gc(); return null;
        }
    }
    static void check(boolean value, String message) {
        if (!value) throw new IllegalStateException(message);
    }
    static int color(Context context) {
        TypedArray a=context.obtainStyledAttributes(R.style.ThemeText,new int[]{android.R.attr.textColor});
        int value=a.getColor(0,0); a.recycle(); return value;
    }
    public static ContextThemeWrapper run() {
        Base base=new Base(); base.resources=0; base.calls=0;
        Themed context=new Themed(base,R.style.PaletteAlternate);
        check(context.getBaseContext()==base && context.applies==0,"lazy construction");
        Resources resources=context.getResources();
        check(base.resources==1 && context.getResources()==resources && base.resources==1,"resource cache/identity");
        check(context.getAssets()==base.getAssets() && context.getApplicationContext()==base.getApplicationContext()
            && context.getClassLoader()==base.getClassLoader() && context.getPackageName().equals(base.getPackageName()),"base delegation");
        Resources.Theme theme=context.getTheme();
        check(theme!=base.theme && context.getTheme()==theme && base.calls==1 && context.applies==1 && context.first,"lazy theme copy/callback");
        check(color(context)==0xff778899 && color(base)==0xff224466,"theme isolation");
        TypedArray old=context.obtainStyledAttributes(R.style.ThemeText,new int[]{android.R.attr.textColor});
        context.setTheme(R.style.PaletteTheme);
        check(context.getTheme()==theme && context.applies==2 && !context.first && color(context)==0xff224466
            && old.getColor(0,0)==0xff778899,"mutable theme/immutable array");
        old.recycle(); base.theme.applyStyle(R.style.PaletteAlternate,true);
        check(color(base)==0xff778899 && color(context)==0xff224466,"base mutation isolation");
        Resources.Theme blank=resources.newTheme();
        TypedArray empty=blank.obtainStyledAttributes(new int[]{R.attr.testPalette});
        check(empty.getColorStateList(0)==null,"newTheme starts empty"); empty.recycle();
        blank.setTo(theme); blank.applyStyle(R.style.PaletteAlternate,true);
        check(color(context)==0xff224466,"Theme.setTo copies styles");
        blank.setTo(resources.newTheme());
        empty=blank.obtainStyledAttributes(new int[]{R.attr.testPalette});
        check(empty.getColorStateList(0)==null,"Theme.setTo replaces styles"); empty.recycle();
        Factory factory=new Factory(); LayoutInflater original=LayoutInflater.from(base); original.setFactory(factory);
        int calls=base.services;
        LayoutInflater clone=LayoutInflater.from(context);
        check(base.services==calls+1 && clone!=original && clone.getContext()==context && clone.getFactory()==factory
            && LayoutInflater.from(context)==clone && base.services==calls+1,"cached themed inflater/factory");
        View leaf=clone.inflate(R.layout.factory_leaf,null);
        check(factory.seen==context && leaf.getContext()==context,"factory uses wrapper context");
        check(context.getSystemService("probe")==base.service,"virtual service delegation");
        base.serviceFault=true;
        try { context.getSystemService("probe"); throw new AssertionError("service fault ignored"); }
        catch (IllegalStateException expected) { check("service failed".equals(expected.getMessage()),"service fault"); }
        base.serviceFault=false;
        check(context.getSystemService("probe")==base.service,"service recovery");
        Themed late=new Themed(); check(late.getBaseContext()==null,"late attachment");
        try { late.getResources(); throw new AssertionError("null base accepted"); } catch (NullPointerException expected) {}
        late.attach(base);
        try { late.attach(base); throw new AssertionError("second attachment accepted"); } catch (IllegalStateException expected) {}
        check(color(late)==0xff778899,"default theme inherits base");
        Themed broken=new Themed(base,R.style.PaletteTheme); base.resourceFault=true;
        try { broken.getTheme(); throw new AssertionError("resource fault ignored"); }
        catch (IllegalStateException expected) { check("resources failed".equals(expected.getMessage()),"resource fault"); }
        base.resourceFault=false; base.fail=true;
        try { broken.getTheme(); throw new AssertionError("theme fault ignored"); }
        catch (IllegalStateException expected) { check("theme callback failed".equals(expected.getMessage()),"theme fault"); }
        base.fail=false;
        check(broken.getTheme()!=null && broken.applies==0,"cached theme after failed initialization");
        broken.fail=true;
        try { broken.setTheme(R.style.PaletteTheme); throw new AssertionError("apply fault ignored"); }
        catch (IllegalStateException expected) { check("apply failed".equals(expected.getMessage()),"apply fault"); }
        broken.fail=false; broken.setTheme(R.style.PaletteTheme);
        check(broken.applies==2 && !broken.first && color(broken)==0xff224466,"apply recovery");
        ContextWrapper plain=new ContextWrapper(base);
        check(plain.getTheme()==base.theme && plain.getResources()==resources && plain.getSystemService("probe")==base.service,"plain wrapper delegation");
        try { new ContextWrapper(null).getTheme(); throw new AssertionError("null wrapper accepted"); } catch (NullPointerException expected) {}
        System.gc(); return context;
    }
}
