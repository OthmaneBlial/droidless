package org.droidless.images;

import android.animation.Animator;
import android.animation.AnimatorListenerAdapter;
import android.app.Activity;
import android.content.pm.ApplicationInfo;
import android.widget.OverScroller;
import android.view.animation.Interpolator;
import android.view.View;
import android.graphics.drawable.Drawable;
import android.graphics.drawable.ColorDrawable;

/** Authored widget contracts, not independent APK compatibility evidence. */
public final class WidgetProbe {
    private static Activity host;
    private static OverScroller scroll;
    public static void verify(Activity activity) {
        host = activity;
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
}
