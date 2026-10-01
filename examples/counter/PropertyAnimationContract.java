package org.droidless.counter;

import android.animation.Animator;
import android.animation.AnimatorListenerAdapter;
import android.animation.TimeInterpolator;
import android.animation.ValueAnimator;
import android.app.Activity;
import android.view.View;
import android.view.ViewPropertyAnimator;
import android.widget.FrameLayout;

/** Authored APK checks for the clock-driven translation/alpha profile. */
public final class PropertyAnimationContract {
    private static FrameLayout root;
    private static View target, other;
    private static Animator last;
    private static String events = "";
    private static int updates, failure;
    private static boolean cancelAtStart, cancelAtCancel, restartAtEnd;
    static void check(boolean value) {
        if (!value) throw new IllegalStateException("Property animation contract failed");
    }
    static final class Listener extends AnimatorListenerAdapter {
        @Override public void onAnimationStart(Animator animator) {
            System.gc();
            last = animator;
            events += "S";
            check(animator instanceof ValueAnimator && animator.isStarted() && target.hasTransientState());
            if (failure == 1) throw new IllegalStateException("property start failure");
            if (cancelAtStart) { cancelAtStart = false; target.animate().cancel(); }
        }
        @Override public void onAnimationCancel(Animator animator) {
            System.gc();
            events += "C";
            check(target.hasTransientState());
            if (failure == 4) throw new IllegalStateException("property cancel failure");
            if (cancelAtCancel) { cancelAtCancel = false; target.animate().cancel(); }
        }
        @Override public void onAnimationEnd(Animator animator) {
            System.gc();
            events += "E";
            check(!animator.isRunning() && !animator.isStarted());
            if (failure == 3) throw new IllegalStateException("property end failure");
            if (restartAtEnd) {
                restartAtEnd = false;
                target.animate().translationY(40).setDuration(50).start();
            }
        }
    }
    static final class Update implements ValueAnimator.AnimatorUpdateListener {
        @Override public void onAnimationUpdate(ValueAnimator animator) {
            System.gc();
            float fraction = animator.getAnimatedFraction();
            check(fraction >= 0 && fraction <= 1 && animator.getDuration() >= 0);
            updates++;
            if (failure == 2) throw new IllegalStateException("property update failure");
        }
    }
    static final class Curve implements TimeInterpolator {
        @Override public float getInterpolation(float fraction) {
            System.gc();
            if (failure == 5) throw new IllegalStateException("property curve failure");
            return fraction;
        }
    }
    static final class Square implements TimeInterpolator {
        @Override public float getInterpolation(float fraction) { System.gc(); return fraction * fraction; }
    }
    public static View setup(Activity activity) {
        root = new FrameLayout(activity);
        target = new View(activity);
        other = new View(activity);
        root.addView(target, new FrameLayout.LayoutParams(50, 30));
        root.addView(other, new FrameLayout.LayoutParams(10, 10));
        ViewPropertyAnimator animator = target.animate();
        check(animator == target.animate() && animator != other.animate());
        check(animator.getDuration() == 300 && animator.getStartDelay() == 0);
        check(animator.getInterpolator() instanceof android.view.animation.AccelerateDecelerateInterpolator);
        try { animator.setDuration(-1); check(false); } catch (IllegalArgumentException expected) {}
        try { animator.setStartDelay(-1); check(false); } catch (IllegalArgumentException expected) {}
        check(animator.getDuration() == 300 && animator.getStartDelay() == 0);
        animator.setInterpolator(null);
        check(animator.getInterpolator() == null);
        animator.setInterpolator(new Curve()).setListener(new Listener()).setUpdateListener(new Update());
        System.gc();
        return root;
    }
    public static void reset() {
        failure = 0;
        cancelAtStart = cancelAtCancel = restartAtEnd = false;
        target.animate().setListener(null).cancel();
        target.setAlpha(1);
        target.setTranslationX(0);
        target.setTranslationY(0);
        target.animate().setStartDelay(0).setDuration(100).setInterpolator(new Curve())
            .setListener(new Listener()).setUpdateListener(new Update());
        events = "";
        updates = 0;
        last = null;
    }
    public static void basic() { target.animate().translationX(100).alpha(0).start(); }
    public static void automatic() { target.animate().translationY(60).setDuration(120); }
    public static void delayed() { target.animate().translationY(100).setStartDelay(50).start(); }
    public static void cancel() { target.animate().cancel(); }
    public static void cancelPending() { target.animate().translationY(90).cancel(); }
    public static void replaceX() { target.animate().translationX(200).start(); }
    public static void recurve() {
        last.setInterpolator(new Square());
        check(((ValueAnimator)last).getInterpolator() instanceof Square);
        check(target.animate().getInterpolator() instanceof Curve);
    }
    public static void cancelReentrantly() {
        cancelAtStart = cancelAtCancel = true;
        target.animate().translationY(80).start();
    }
    public static void restartOnEnd() {
        restartAtEnd = true;
        target.animate().translationY(80).setDuration(50).start();
    }
    public static void fail(int kind) {
        failure = kind;
        target.animate().translationX(100).start();
    }
    public static void failAtCancel() { failure = 4; cancel(); }
    public static void clearFailure() { failure = 0; }
    public static String events() { return events; }
    public static int updates() { return updates; }
    public static float x() { return target.getTranslationX(); }
    public static float y() { return target.getTranslationY(); }
    public static float alpha() { return target.getAlpha(); }
    public static boolean transientState() { return target.hasTransientState(); }
    public static boolean ended() { return last != null && !last.isStarted() && !last.isRunning(); }
    public static void release() { root = null; target = other = null; last = null; }
}
