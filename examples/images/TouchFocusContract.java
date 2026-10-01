package org.droidless.images;

import android.app.Activity;
import android.view.MotionEvent;
import android.view.View;
import android.widget.EditText;

/** First touch takes focus; only subsequent taps dispatch a click. */
public final class TouchFocusContract {
    static int gains, clicks;
    static void check(boolean value) {
        if (!value) throw new IllegalStateException("touch focus contract");
    }
    static void tap(View view, boolean cancel) {
        MotionEvent down = MotionEvent.obtain(0, 0, MotionEvent.ACTION_DOWN, 20, 20, 0);
        MotionEvent end = MotionEvent.obtain(0, 1, cancel ? MotionEvent.ACTION_CANCEL : MotionEvent.ACTION_UP, 20, 20, 0);
        check(view.dispatchTouchEvent(down)); check(view.dispatchTouchEvent(end));
        down.recycle(); end.recycle();
    }
    public static int run(Activity context) {
        gains = clicks = 0;
        EditText editor = new EditText(context);
        check(editor.isClickable() && !editor.isFocused());
        editor.setOnFocusChangeListener(new View.OnFocusChangeListener() {
            public void onFocusChange(View view, boolean focused) {
                System.gc(); check(view.isFocused() == focused); if (focused) gains++;
            }
        });
        editor.setOnClickListener(new View.OnClickListener() {
            public void onClick(View view) { System.gc(); clicks++; }
        });
        tap(editor, true); check(!editor.isFocused() && gains == 0 && clicks == 0);
        tap(editor, false); check(editor.isFocused() && gains == 1 && clicks == 0);
        tap(editor, false); check(gains == 1 && clicks == 1);
        editor.setFocusable(false); editor.setFocusable(true);
        tap(editor, false); check(!editor.isFocused() && clicks == 2);
        editor.setFocusableInTouchMode(true); editor.setEnabled(false);
        tap(editor, false); check(!editor.isFocused() && clicks == 2);
        return 1;
    }
}
