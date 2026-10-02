package org.droidless.images;

import android.app.Activity;
import android.graphics.drawable.ColorDrawable;
import android.graphics.drawable.Drawable;
import android.graphics.drawable.StateListDrawable;
import android.view.View;

/** Ordered state selection, retained arrays, guest callbacks and rendered backgrounds. */
public final class SelectorDrawableContract {
    static final int PRESSED=16842919, ENABLED=16842910;
    static class Selector extends StateListDrawable {
        int callbacks;
        boolean fail;
        protected boolean onStateChange(int[] states) {
            callbacks++; System.gc();
            if (fail) throw new IllegalStateException("selector callback failed");
            return super.onStateChange(states);
        }
    }
    static class Color extends ColorDrawable {
        int callbacks;
        Color(int color) { super(color); }
        public boolean setState(int[] states) {
            callbacks++; System.gc(); return super.setState(states);
        }
    }
    static void check(boolean okay,String message) {
        if (!okay) throw new IllegalStateException(message);
    }
    public static View run(Activity activity) {
        Selector selector=new Selector();
        Color pressed=new Color(0xff1687ff), disabled=new Color(0xff444444), normal=new Color(0xff112233);
        int[] spec={PRESSED,0,ENABLED};
        selector.addState(spec,pressed);
        selector.addState(new int[]{-ENABLED},disabled);
        selector.addState(new int[0],normal);
        check(selector.isStateful() && selector.getCurrent()==disabled,"negative state selection");
        check(selector.getIntrinsicWidth()==-1 && selector.getIntrinsicHeight()==-1 && selector.getMinimumWidth()==0,"current child intrinsic size");
        int[] state={PRESSED,0}; selector.setState(state);
        check(selector.getCurrent()==pressed && pressed.callbacks>0,"positive zero-terminated selection");
        int calls=selector.callbacks;
        check(!selector.setState(new int[]{PRESSED,0}) && selector.getState()==state && selector.callbacks==calls,"equal state retains original");
        spec[0]=-PRESSED; selector.setState(new int[]{ENABLED});
        check(selector.getCurrent()==pressed,"retained specification array");
        spec[0]=PRESSED; selector.setState(new int[]{ENABLED,PRESSED});
        selector.setState(new int[]{ENABLED}); check(selector.getCurrent()==normal,"wildcard fallback");
        selector.addState(null,null); check(selector.getCurrent()==normal,"null child ignored");
        Selector first=new Selector(); first.addState(new int[0],normal); first.addState(new int[]{PRESSED},pressed);
        first.setState(new int[]{PRESSED}); check(first.getCurrent()==normal,"ordered first match");
        StateListDrawable empty=new StateListDrawable(); check(empty.getCurrent()==null && empty.getIntrinsicWidth()==-1 && empty.getIntrinsicHeight()==-1,"empty selector");
        Selector nested=new Selector(); nested.addState(new int[0],selector);
        View view=new View(activity); view.setBackground(nested);
        check(selector.getCurrent()==normal,"initial view state");
        view.setPressed(true); check(selector.getCurrent()==pressed,"view pressed state");
        view.setPressed(false); view.setEnabled(false); check(selector.getCurrent()==disabled,"view disabled state");
        view.setEnabled(true); activity.setContentView(view);
        System.gc(); check(view.getBackground()==nested && selector.getCurrent()==normal,"background roots");
        return view;
    }
    public static void fault(View view,boolean fail) {
        Selector nested=(Selector)view.getBackground(); nested.fail=fail;
        view.setPressed(fail);
    }
}
