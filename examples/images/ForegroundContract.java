package org.droidless.images;
import android.app.Activity;
import android.graphics.Rect;
import android.graphics.drawable.ColorDrawable;
import android.graphics.drawable.Drawable;
import android.graphics.drawable.StateListDrawable;
import android.view.Gravity;
import android.view.View;
import android.widget.Button;
import android.widget.FrameLayout;

public final class ForegroundContract {
    static FrameLayout frame;
    static ColorDrawable color;
    static StateListDrawable selector;
    static Drawable retained;
    static int clicks, paddingCalls;
    static boolean failPadding;
    static final class Padded extends ColorDrawable {
        Padded(int color) { super(color); }
        @Override public boolean getPadding(Rect out) {
            paddingCalls++; System.gc();
            if (failPadding) throw new IllegalStateException("foreground padding failure");
            out.set(9, 11, 13, 15); return true;
        }
    }
    static void check(boolean value, String reason) { if (!value) throw new IllegalStateException(reason); }
    public static View begin(Activity activity) {
        frame = new FrameLayout(activity); frame.setPadding(3, 4, 20, 6);
        Button child = new Button(activity); child.setText("Through foreground");
        child.setOnClickListener(new View.OnClickListener() { public void onClick(View view) { clicks++; } });
        frame.addView(child, new FrameLayout.LayoutParams(100, 50));
        activity.setContentView(frame);
        check(frame.getForeground() == null && frame.getForegroundGravity() == Gravity.FILL, "initial foreground");
        frame.setForegroundGravity(Gravity.TOP | Gravity.FILL_HORIZONTAL);
        check(frame.getForegroundGravity() == (Gravity.TOP | Gravity.FILL_HORIZONTAL), "null foreground gravity");
        frame.setForegroundGravity(Gravity.FILL);
        color = new Padded(0x80224466); frame.setForeground(color);
        check(frame.getForeground() == color && color.getCallback() == frame, "foreground ownership/callback");
        check(frame.getPaddingLeft() == 3 && frame.getPaddingRight() == 20, "foreground did not overwrite user padding");
        frame.setForeground(color); check(paddingCalls == 1, "same foreground is a no-op");
        return frame;
    }
    public static void visible(boolean visible) { color.setVisible(visible, false); }
    public static void recolor() { color.setColor(0x80446688); }
    public static void select() {
        selector = new StateListDrawable();
        selector.addState(new int[]{android.R.attr.state_pressed}, new ColorDrawable(0x806699aa));
        selector.addState(new int[0], new ColorDrawable(0x80335577));
        frame.setForeground(selector); check(color.getCallback() == null, "replacement detaches old callback");
        check(selector.getCallback() == frame, "selector callback");
    }
    public static void press(boolean pressed) { frame.setPressed(pressed); }
    public static void clear() { frame.setForeground(null); check(selector.getCallback() == null, "clear callback"); }
    public static int clicks() { return clicks; }
    public static void fail() { failPadding = true; frame.setForeground(color); }
    public static void recover() { failPadding = false; frame.setForeground(null); frame.setForeground(color); }
    public static Drawable orphan(Activity activity) {
        FrameLayout orphan = new FrameLayout(activity); ColorDrawable value = new ColorDrawable(0x80224466); retained = value;
        orphan.setForeground(value); return value;
    }
    public static boolean cleared(Drawable drawable) { return drawable.getCallback() == null; }
    public static void drop() { frame = null; selector = null; color = null; retained = null; }
}
