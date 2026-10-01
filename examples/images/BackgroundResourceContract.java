package org.droidless.images;

import android.app.Activity;
import android.content.Context;
import android.content.ContextWrapper;
import android.graphics.drawable.Drawable;
import android.graphics.drawable.ColorDrawable;
import android.view.View;

/** Actual context/background callbacks, resource caching and GC/fault recovery. */
public final class BackgroundResourceContract {
    static class ResourcesContext extends ContextWrapper {
        final ProbeResources resources;
        ResourcesContext(Context context) { super(context); resources = new ProbeResources(context); }
        public android.content.res.Resources getResources() { return resources; }
    }
    static class ProbeResources extends android.content.res.Resources {
        int loads;
        ProbeResources(Context context) {
            super(context.getAssets(), context.getResources().getDisplayMetrics(), context.getResources().getConfiguration());
        }
        public Drawable getDrawable(int id, Theme theme) {
            loads++; System.gc();
            if (id==13) throw new IllegalStateException("drawable lookup failed");
            return new ColorDrawable(id);
        }
    }
    static class Probe extends View {
        int backgrounds, drawables;
        boolean fail;
        Probe(Context context) { super(context); }
        public void setBackground(Drawable drawable) {
            backgrounds++; System.gc(); super.setBackground(drawable);
        }
        public void setBackgroundDrawable(Drawable drawable) {
            drawables++; System.gc();
            if (fail) throw new IllegalStateException("background callback failed");
            super.setBackgroundDrawable(drawable);
        }
    }
    static void check(boolean value) { if (!value) throw new IllegalStateException("background resource contract"); }
    public static View run(Activity host) {
        ResourcesContext context = new ResourcesContext(host); Probe view = new Probe(context);
        view.setBackgroundResource(42); System.gc(); Drawable first = view.getBackground();
        check(first instanceof ColorDrawable && ((ColorDrawable)first).getColor()==42);
        view.setBackgroundResource(42);
        check(context.resources.loads==1 && view.backgrounds==1 && view.drawables==1 && view.getBackground()==first);
        view.setBackground(first); view.setBackgroundResource(42);
        check(context.resources.loads==1 && view.backgrounds==2 && view.drawables==2);
        view.setBackground(new ColorDrawable(7)); view.setBackgroundResource(42);
        check(context.resources.loads==2 && view.getBackground()!=first);
        view.setBackgroundColor(9); view.setBackgroundResource(42); check(context.resources.loads==3);
        Drawable retained = view.getBackground();
        try { view.setBackgroundResource(13); throw new IllegalStateException("lookup fault missing"); }
        catch (IllegalStateException expected) { check("drawable lookup failed".equals(expected.getMessage()) && view.getBackground()==retained); }
        view.setBackgroundResource(0); view.setBackgroundResource(0); check(view.getBackground()==null && context.resources.loads==4);
        view.fail=true;
        try { view.setBackgroundResource(42); throw new IllegalStateException("callback fault missing"); }
        catch (IllegalStateException expected) { check("background callback failed".equals(expected.getMessage()) && view.getBackground()==null); }
        view.fail=false; view.setBackgroundResource(42); check(context.resources.loads==6 && view.getBackground()!=null);
        return view;
    }
}
