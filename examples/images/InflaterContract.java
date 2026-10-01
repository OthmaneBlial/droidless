package org.droidless.images;

import android.app.Activity;
import android.content.Context;
import android.content.ContextWrapper;
import android.util.AttributeSet;
import android.view.LayoutInflater;
import android.view.View;
import android.view.ViewGroup;
import android.view.ViewStub;
import android.widget.FrameLayout;
import android.widget.ImageButton;

/** Authored inflater contracts; independent APK evidence is recorded separately. */
public final class InflaterContract {
    static Activity host;
    static LayoutInflater original, clone, deep;
    static View tree;
    static Factory oldFactory, newFactory;
    static String order = "";
    static int constructed, finished;
    static class Probe extends ImageButton {
        final AttributeSet receivedAttrs;
        final int attributeId;
        Probe(Context context, AttributeSet attrs) {
            super(context, attrs); receivedAttrs = attrs;
            attributeId = attrs.getAttributeResourceValue("http://schemas.android.com/apk/res/android", "id", 0);
            constructed++; System.gc();
        }
        protected void onFinishInflate() { super.onFinishInflate(); finished++; System.gc(); }
    }
    static class Factory implements LayoutInflater.Factory2 {
        final String label;
        boolean produce, fail;
        int calls3, calls4;
        Context context;
        View parent;
        AttributeSet attrs;
        View firstParent;
        Factory(String label, boolean produce) { this.label = label; this.produce = produce; }
        public View onCreateView(String name, Context context, AttributeSet attrs) {
            calls3++; return create(null, name, context, attrs);
        }
        public View onCreateView(View parent, String name, Context context, AttributeSet attrs) {
            calls4++; return create(parent, name, context, attrs);
        }
        View create(View parent, String name, Context context, AttributeSet attrs) {
            this.parent = parent; this.context = context; this.attrs = attrs;
            if (calls3 + calls4 == 1) firstParent = parent;
            order += label + ":" + name + ";";
            System.gc();
            if (fail) throw new IllegalStateException("inflater callback failed");
            if (!produce || !name.equals("ImageButton")) return null;
            return new Probe(context, attrs);
        }
        void release() { parent = null; firstParent = null; context = null; attrs = null; }
    }
    static class RoutedContext extends ContextWrapper {
        final LayoutInflater inflater;
        RoutedContext(Context base, LayoutInflater inflater) { super(base); this.inflater = inflater; }
        public Object getSystemService(String name) {
            return name.equals(Context.LAYOUT_INFLATER_SERVICE) ? inflater : super.getSystemService(name);
        }
    }
    static void check(boolean value, String message) { if (!value) throw new IllegalStateException(message); }
    public static int run(Activity activity) {
        host = activity;
        check(new android.widget.TextView(activity).getTransformationMethod() == null, "default untransformed TextView");
        original = LayoutInflater.from(activity);
        check(original == activity.getLayoutInflater()
            && original == activity.getSystemService(Context.LAYOUT_INFLATER_SERVICE), "inflater service identity");
        check(original.getContext() == activity && original.getFactory() == null
            && original.getFactory2() == null, "initial inflater state");
        try { LayoutInflater.from(new RoutedContext(activity, null)); throw new IllegalStateException("missing service accepted"); }
        catch (AssertionError expected) {}
        try { original.setFactory2(null); throw new AssertionError("null factory accepted"); }
        catch (NullPointerException expected) {}
        oldFactory = new Factory("old", true);
        original.setFactory2(oldFactory);
        check(original.getFactory() == oldFactory && original.getFactory2() == oldFactory, "Factory2 identity");
        try { original.setFactory(oldFactory); throw new AssertionError("second factory accepted"); }
        catch (IllegalStateException expected) {}
        FrameLayout parent = new FrameLayout(activity);
        tree = original.inflate(R.layout.factory_main, parent, false);
        check(tree.getParent() == null && tree.getLayoutParams() != null, "detached root parameters");
        check(oldFactory.firstParent == parent, "root factory receives detached parent");
        check(tree.findViewById(R.id.factory_button) instanceof Probe
            && tree.findViewById(R.id.factory_included) instanceof Probe, "factory substitution/include");
        check(oldFactory.calls3 == 0 && oldFactory.calls4 == 4, "Factory2 callback selection");
        Probe button = (Probe)tree.findViewById(R.id.factory_button);
        check(button.attributeId == R.id.factory_button && button.receivedAttrs != null
            && button.getContext() == activity, "factory receives real XML attributes and context");
        ViewStub stub = (ViewStub)tree.findViewById(R.id.factory_stub);
        check(stub.getLayoutInflater() != original && stub.getLayoutInflater().getFactory2() == oldFactory,
            "ViewStub captures cloned factory state");
        View inflated = stub.inflate();
        check(inflated instanceof Probe && oldFactory.parent == tree
            && inflated.getParent() == tree, "ViewStub factory and parent");
        check(constructed == 3 && finished == 3, "single constructor and finish callbacks");
        activity.setContentView(R.layout.factory_main);
        check(activity.findViewById(R.id.factory_button) instanceof Probe, "Activity uses installed factory");
        Context other = activity.getApplicationContext();
        clone = original.cloneInContext(other);
        check(clone != original && clone.getContext() == other && clone.getFactory2() == oldFactory, "clone state");
        check(LayoutInflater.from(new RoutedContext(activity, clone)) == clone, "virtual service routing");
        newFactory = new Factory("new", false);
        clone.setFactory2(newFactory);
        check(clone.getFactory() == clone.getFactory2() && clone.getFactory2() instanceof LayoutInflater.Factory2
            && clone.getFactory2() != oldFactory, "merged factory identity/type");
        order = "";
        View child = clone.inflate(R.layout.factory_leaf, parent, false);
        check(child instanceof Probe && oldFactory.context == other && oldFactory.parent == parent
            && order.equals("new:ImageButton;old:ImageButton;"), "clone fallback/order/context/parent");
        check(((Probe)child).receivedAttrs == oldFactory.attrs, "callback/constructor attribute identity");
        ViewStub secondStub = (ViewStub)activity.findViewById(R.id.factory_stub);
        secondStub.setLayoutInflater(clone);
        check(secondStub.inflate().getContext() == other, "explicit ViewStub inflater");
        LayoutInflater firstOnly = original.cloneInContext(other);
        Factory first = new Factory("first", true);
        firstOnly.setFactory(first);
        check(firstOnly.getFactory2() == oldFactory, "Factory setter preserves inherited Factory2");
        child = firstOnly.inflate(R.layout.factory_leaf, null);
        check(child instanceof Probe && first.calls3 == 0, "Factory2 priority");
        check(firstOnly.getFactory().onCreateView("ImageButton", other, oldFactory.attrs) instanceof Probe
            && first.calls3 == 1, "merger three-argument dispatch");
        LayoutInflater plain = LayoutInflater.from(other);
        Factory legacy = new Factory("legacy", true);
        plain.setFactory(legacy);
        check(plain.getFactory2() == null && plain.inflate(R.layout.factory_leaf, null) instanceof Probe
            && legacy.calls3 == 1 && legacy.calls4 == 0, "legacy Factory dispatch");
        System.gc();
        return 1;
    }
    public static int verifyDeepMerge() {
        deep = original;
        for (int i = 0; i < 64; i++) {
            deep = deep.cloneInContext(host);
            deep.setFactory2(new Factory("limit", false));
        }
        return deep.inflate(R.layout.factory_leaf, null) instanceof Probe ? 1 : 0;
    }
    public static void mergeLimit() {
        deep.cloneInContext(host).setFactory2(new Factory("beyond", false));
    }
    public static LayoutInflater deepRoot() { return deep; }
    public static void callbackFault() {
        newFactory.fail = true;
        try { clone.inflate(R.layout.factory_leaf, null); }
        finally { newFactory.fail = false; }
    }
    public static int recover() { return clone.inflate(R.layout.factory_leaf, null) instanceof Probe ? 1 : 0; }
    public static void startWorker() {
        new Thread(new Runnable() { public void run() { clone.inflate(R.layout.factory_leaf, null); } }).start();
    }
    public static View root() { return tree; }
    public static void release() {
        host.setContentView(new View(host));
        oldFactory.release(); newFactory.release();
        original = null; clone = null; deep = null; tree = null; host = null; oldFactory = null; newFactory = null;
    }
}
