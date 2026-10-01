package org.droidless.images;

import android.animation.Animator;
import android.animation.AnimatorListenerAdapter;
import android.app.Activity;
import android.content.pm.ApplicationInfo;
import android.widget.OverScroller;
import android.widget.FrameLayout;
import android.view.animation.Interpolator;
import android.view.View;
import android.graphics.drawable.Drawable;
import android.graphics.drawable.ColorDrawable;

/** Authored widget contracts, not independent APK compatibility evidence. */
public final class WidgetProbe {
    private static Activity host;
    private static OverScroller scroll;
    private static FrameLayout frameRoot;
    private static ScrollFrame frame, hidden;
    public static void verify(Activity activity) {
        host = activity;
        android.content.res.Configuration config = activity.getResources().getConfiguration();
        android.content.res.Configuration copied = new android.content.res.Configuration(config);
        System.gc();
        if (config==copied || !config.equals(copied) || !copied.equals((Object)config)
            || config.equals((android.content.res.Configuration)null) || config.equals(new Object()))
            throw new IllegalStateException("profile configuration equality");
        copied.keyboard=config.keyboard+1;
        if (config.equals(copied) || copied.equals(config)) throw new IllegalStateException("configuration mutation");
        copied.keyboard=config.keyboard;
        if (!config.equals(copied)) throw new IllegalStateException("configuration field aliases");
        android.content.res.Configuration fresh = new android.content.res.Configuration();
        if (fresh.fontScale!=1 || fresh.keyboard!=0 || config.fontScale!=1 || config.densityDpi!=160)
            throw new IllegalStateException("configuration defaults and readable profile fields");
        copied.fontScale=2;
        if (config.equals(copied)) throw new IllegalStateException("configuration float change");
        copied.fontScale=config.fontScale;
        fresh.fontScale=-0.0f; copied=new android.content.res.Configuration(fresh); copied.fontScale=0.0f;
        if (!fresh.equals(copied)) throw new IllegalStateException("configuration signed zero");
        fresh.fontScale=Float.NaN;
        if (!fresh.equals(copied)) throw new IllegalStateException("configuration API21 NaN comparison");
        int enabled=16842910, pressed=16842919;
        int[][] specs={{pressed,0,enabled},{-enabled},{}};
        int[] colors={0xff112233,0x80112233,0xff778899};
        android.content.res.ColorStateList palette = new android.content.res.ColorStateList(specs,colors);
        System.gc();
        if (!palette.isStateful() || palette.getDefaultColor()!=colors[2]
            || palette.getColorForState(new int[]{pressed},7)!=colors[0]
            || palette.getColorForState(new int[]{},7)!=colors[1]
            || palette.getColorForState(new int[]{enabled},7)!=colors[2]
            || palette.getColorForState(null,7)!=colors[2]) throw new IllegalStateException("state color matching");
        colors[0]=0xffabcdef; colors[2]=0xff998877; System.gc();
        if (palette.getColorForState(new int[]{pressed},7)!=colors[0]
            || palette.getDefaultColor()!=0xff778899) throw new IllegalStateException("live colors and cached default");
        android.content.res.ColorStateList empty = new android.content.res.ColorStateList(new int[][]{},new int[]{});
        android.content.res.ColorStateList single = new android.content.res.ColorStateList(new int[][]{{pressed}},new int[]{42});
        if (empty.isStateful() || empty.getDefaultColor()!=0xffff0000 || empty.getColorForState(new int[]{enabled},7)!=7
            || single.isStateful() || single.getColorForState(null,7)!=7) throw new IllegalStateException("empty/single palette");
        final int[] actions = {0};
        android.widget.TextView editor = new android.widget.TextView(activity);
        editor.setHintTextColor(palette); editor.setLinkTextColor(palette); System.gc();
        if (editor.getHintTextColors()!=palette || editor.getLinkTextColors()!=palette)
            throw new IllegalStateException("hint/link color identity");
        editor.setHintTextColor((android.content.res.ColorStateList)null);
        editor.setLinkTextColor((android.content.res.ColorStateList)null);
        if (editor.getHintTextColors()!=null || editor.getLinkTextColors()!=null)
            throw new IllegalStateException("clear hint/link colors");
        editor.setOnEditorActionListener(new android.widget.TextView.OnEditorActionListener() {
            public boolean onEditorAction(android.widget.TextView view, int id, android.view.KeyEvent event) {
                System.gc();
                if (event != null) throw new IllegalStateException("editor action key should be null");
                actions[0] += id;
                return id == 6;
            }
        });
        System.gc(); editor.onEditorAction(6); editor.onEditorAction(3);
        editor.setOnEditorActionListener(null); System.gc(); editor.onEditorAction(6);
        if (actions[0] != 9) throw new IllegalStateException("editor action delivery and clearing");
        if (android.graphics.Color.alpha(0x80ff0088) != 128 || android.graphics.Color.alpha(0) != 0
            || android.graphics.Color.alpha(-1) != 255) throw new IllegalStateException("ARGB alpha extraction");
        android.widget.CheckedTextView checked = new android.widget.CheckedTextView(activity);
        Drawable mark = new ColorDrawable(0xff224466);
        if (checked.getCheckMarkDrawable() != null) throw new IllegalStateException("default checkmark");
        checked.setCheckMarkDrawable(mark); System.gc();
        if (checked.getCheckMarkDrawable() != mark) throw new IllegalStateException("checkmark identity");
        checked.setChecked(true);
        if (!checked.isChecked() || checked.getCheckMarkDrawable() != mark) throw new IllegalStateException("checked state and mark");
        checked.setCheckMarkDrawable(null); System.gc();
        if (checked.getCheckMarkDrawable() != null || !checked.isChecked()) throw new IllegalStateException("clear checkmark");
        int[] states = {16842910, -16842919, 0, 0};
        int[] trimmed = android.util.StateSet.trimStateSet(states, 2);
        System.gc();
        if (trimmed == states || trimmed.length != 2 || trimmed[0] != states[0] || trimmed[1] != states[1]
            || android.util.StateSet.trimStateSet(states, 4) != states
            || android.util.StateSet.trimStateSet(states, 0).length != 0)
            throw new IllegalStateException("StateSet prefix or identity");
        trimmed[0] = 7;
        if (states[0] != 16842910) throw new IllegalStateException("StateSet copy aliases input");
        try { android.util.StateSet.trimStateSet(null, 0); throw new IllegalStateException("null states accepted"); }
        catch (NullPointerException expected) {}
        try { android.util.StateSet.trimStateSet(states, -1); throw new IllegalStateException("negative size accepted"); }
        catch (NegativeArraySizeException expected) {}
        try { android.util.StateSet.trimStateSet(states, 5); throw new IllegalStateException("large size accepted"); }
        catch (IndexOutOfBoundsException expected) {}
        android.content.res.Resources resources = activity.getResources();
        int[] resourceIds = {R.layout.main, R.id.source, R.style.ProbeText, R.drawable.sample_alias};
        String[] entryNames = {"main", "source", "ProbeText", "sample_alias"};
        for (int i = 0; i < resourceIds.length; i++) {
            String name = resources.getResourceEntryName(resourceIds[i]);
            System.gc();
            if (!name.equals(entryNames[i])) throw new AssertionError("resource entry name");
        }
        for (int id : new int[] {0, -1, 0x7fffffff, 0x01030000}) {
            try {
                resources.getResourceEntryName(id);
                throw new AssertionError("missing resource accepted");
            } catch (RuntimeException error) {
                System.gc();
                if (!(error instanceof android.content.res.Resources.NotFoundException)
                        || error.getMessage() == null || error.getMessage().length() == 0)
                    throw new AssertionError("resource lookup fault");
            }
        }
        final int[] calls = {0};
        AnimatorListenerAdapter adapter = new AnimatorListenerAdapter() {
            public void onAnimationEnd(Animator animation) { super.onAnimationEnd(animation); calls[0]++; }
        };
        Object object = adapter;
        if (!(object instanceof Animator.AnimatorListener) || !(object instanceof Animator.AnimatorPauseListener))
            throw new AssertionError("adapter interfaces");
        Animator.AnimatorListener listener = adapter;
        listener.onAnimationStart(null); listener.onAnimationRepeat(null); listener.onAnimationCancel(null);
        listener.onAnimationEnd(null);
        Animator.AnimatorPauseListener pause = adapter;
        pause.onAnimationPause(null); pause.onAnimationResume(null);
        System.gc();
        if (calls[0] != 1) throw new AssertionError("adapter virtual callback");
        ApplicationInfo info = activity.getApplicationInfo();
        if (info.targetSdkVersion != 28 || !info.packageName.equals("org.droidless.images")
                || info.name != null || info.labelRes != 0)
            throw new AssertionError("manifest application metadata");
        System.gc();
        if (activity.getApplicationInfo() != info || activity.getApplicationContext().getApplicationInfo() != info)
            throw new AssertionError("application metadata identity");
        try {
            ApplicationInfo copy = activity.getPackageManager().getActivityInfo(activity.getComponentName(), 0).applicationInfo;
            if (copy.targetSdkVersion != info.targetSdkVersion || !copy.packageName.equals(info.packageName))
                throw new AssertionError("ActivityInfo application metadata");
            copy.targetSdkVersion = 1;
            if (activity.getApplicationInfo().targetSdkVersion != 28) throw new AssertionError("ActivityInfo copy");
        } catch (Exception error) { throw new AssertionError(error); }
        OverScroller initial = new OverScroller(activity);
        if (!initial.isFinished() || initial.computeScrollOffset() || initial.getCurrX() != 0 || initial.getFinalY() != 0)
            throw new AssertionError("initial scroller state");
        BackgroundProbe view = new BackgroundProbe(activity);
        Drawable background = new ColorDrawable(0xff202a36);
        view.setBackground(background); System.gc();
        if (view.calls != 1 || view.getBackground() != background) throw new AssertionError("background virtual dispatch");
        view.setBackground(null);
        if (view.calls != 2 || view.getBackground() != null) throw new AssertionError("background clear");
        String description = new StringBuilder().append("Accessible view").toString();
        view.setContentDescription(description); System.gc();
        if (view.getContentDescription() != description) throw new AssertionError("description ownership");
        if (view.getImportantForAccessibility() != View.IMPORTANT_FOR_ACCESSIBILITY_YES) throw new AssertionError("description accessibility importance");
        view.setContentDescription(new StringBuilder().append("Accessible view").toString());
        if (view.getContentDescription() != description) throw new AssertionError("equal description identity");
        view.setContentDescription("");
        if (view.getContentDescription() == null || view.getContentDescription().length() != 0) throw new AssertionError("empty description");
        view.setContentDescription(description); view.setContentDescription(null);
        if (view.getContentDescription() != null) throw new AssertionError("description clear");
    }
    private static class BackgroundProbe extends View {
        int calls;
        BackgroundProbe(Activity context) { super(context); }
        public void setBackgroundDrawable(Drawable background) { calls++; System.gc(); super.setBackgroundDrawable(background); }
    }
    public static void startScroll() {
        scroll = new OverScroller(host, new Interpolator() {
            public float getInterpolation(float input) { System.gc(); return input; }
        });
        scroll.startScroll(10, 20, 21, -41, 1000);
    }
    public static void startDefaultScroll() {
        scroll = new OverScroller(host); scroll.startScroll(0, 0, 1000, -1000);
    }
    public static void startLargeScroll() {
        scroll = new OverScroller(host, new Interpolator() {
            public float getInterpolation(float input) { System.gc(); return 1; }
        });
        scroll.startScroll(0, 0, 16777215, 0, 1);
    }
    public static void startInstantScroll() {
        scroll = new OverScroller(host); scroll.startScroll(1, 2, 3, 4, 0);
    }
    public static int metadataTarget(Activity context) { return context.getApplicationInfo().targetSdkVersion; }
    public static OverScroller failedScroll() {
        scroll = new OverScroller(host, new Interpolator() {
            public float getInterpolation(float input) { System.gc(); throw new IllegalStateException("interpolation failed"); }
        });
        scroll.startScroll(0, 0, 1, 1, 1000);
        try { scroll.computeScrollOffset(); } catch (IllegalStateException expected) { return scroll; }
        throw new AssertionError("interpolation failure swallowed");
    }
    public static void clearScroll() { scroll = null; }
    public static String scrollState() {
        boolean active = scroll.computeScrollOffset();
        return active + ":" + scroll.isFinished() + ":" + scroll.getCurrX() + ":" + scroll.getCurrY();
    }
    public static void stopScroll() { scroll.forceFinished(true); }
    public static void abortScroll() { scroll.abortAnimation(); }
    public static void invalidateFrame() {
        View view = host.findViewById(R.id.source);
        view.computeScroll();
        view.postInvalidateOnAnimation();
        view.postInvalidateOnAnimation();
        System.gc();
    }
    public static void invalidateDetachedFrame() {
        new View(host).postInvalidateOnAnimation();
    }
    public static boolean attachedWindowFocus() { return host.findViewById(R.id.source).hasWindowFocus(); }
    public static boolean detachedWindowFocus() { return new View(host).hasWindowFocus(); }
    private static class ScrollFrame extends View {
        final OverScroller scroller;
        int calls;
        boolean fail, removed;
        View toRemove;
        ScrollFrame(Activity context) {
            super(context);
            scroller = new OverScroller(context, new Interpolator() {
                public float getInterpolation(float input) { System.gc(); return input; }
            });
        }
        @Override public void computeScroll() {
            super.computeScroll();
            if (removed) throw new AssertionError("detached child callback");
            if (fail) throw new IllegalStateException("scroll frame failure");
            calls++;
            if (toRemove != null) { frameRoot.removeView(toRemove); toRemove = null; }
            System.gc();
            if (scroller.computeScrollOffset()) {
                setTranslationX(scroller.getCurrX());
                if (!scroller.isFinished()) postInvalidateOnAnimation();
            }
        }
    }
    public static View startFrame() {
        frameRoot = new FrameLayout(host);
        frame = new ScrollFrame(host);
        hidden = new ScrollFrame(host); hidden.setVisibility(View.INVISIBLE);
        ScrollFrame removed = new ScrollFrame(host); removed.removed = true;
        frame.toRemove = removed;
        frameRoot.addView(frame); frameRoot.addView(hidden); frameRoot.addView(removed);
        host.setContentView(frameRoot);
        frame.scroller.startScroll(0, 0, 100, 0, 1000);
        return removed;
    }
    public static String frameState() { return frame.calls + ":" + (int)frame.getTranslationX() + ":" + hidden.calls; }
    public static View frameRoot() { return frameRoot; }
    public static void failFrame() { frame.fail = true; }
    public static void recoverFrame() { frame.fail = false; }
    public static void releaseFrame() {
        host.setContentView(new View(host)); frameRoot = null; frame = null; hidden = null;
    }
}
