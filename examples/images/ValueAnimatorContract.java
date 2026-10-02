package org.droidless.images;

import android.animation.Animator;
import android.animation.AnimatorListenerAdapter;
import android.animation.ValueAnimator;
import android.view.animation.LinearInterpolator;

/** Scalar animations use the runtime clock and execute real guest listeners. */
public final class ValueAnimatorContract {
    static ValueAnimator value;
    static String events="";
    static boolean fail;
    static void check(boolean condition) {
        if (!condition) throw new IllegalStateException("value animation contract: "+events);
    }
    public static ValueAnimator start() {
        value=new ValueAnimator(); check(value.getDuration()==300 && !value.isRunning());
        check(value.setDuration(1000)==value);
        value.setInterpolator(new LinearInterpolator()); value.setFloatValues(0,10,30);
        value.addUpdateListener(new ValueAnimator.AnimatorUpdateListener() {
            public void onAnimationUpdate(ValueAnimator animation) {
                check(animation==value); events+="U"; System.gc();
                if (fail) throw new IllegalStateException("update failure");
            }
        });
        value.addListener(new AnimatorListenerAdapter() {
            public void onAnimationStart(Animator a) { events+="S"; System.gc(); }
            public void onAnimationCancel(Animator a) { events+="C"; System.gc(); }
            public void onAnimationEnd(Animator a) { events+="E"; System.gc(); }
        });
        events=""; value.start(); check(events.equals("US") && value.isRunning());
        check(((Float)value.getAnimatedValue()).floatValue()==0 && value.getAnimatedFraction()==0);
        return value;
    }
    public static void halfway() {
        check(value.isRunning() && value.getAnimatedFraction()==0.5f);
        check(((Float)value.getAnimatedValue()).floatValue()==10 && events.equals("USU"));
        value.end(); check(!value.isStarted() && ((Float)value.getAnimatedValue()).floatValue()==30);
        check(events.equals("USUUE"));
        value.setStartDelay(100); value.setIntValues(10,20); events=""; value.start();
        check(value.isStarted() && !value.isRunning() && events.equals(""));
    }
    public static void delayed() {
        check(value.isRunning() && ((Integer)value.getAnimatedValue()).intValue()==10);
        check(events.equals("SU")); value.cancel(); check(events.equals("SUCE") && !value.isStarted());
        try { value.setDuration(-1); throw new AssertionError("negative duration accepted"); }
        catch(IllegalArgumentException expected) {}
        value.setStartDelay(0); events=""; fail=true;
        try { value.start(); throw new AssertionError("update fault skipped"); }
        catch(IllegalStateException expected) {}
        check(!value.isStarted()); fail=false; value.start(); value.cancel(); value=null;
    }
}
