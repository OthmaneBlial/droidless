package org.droidless.counter;

import android.app.Activity;
import android.text.Layout;
import android.view.View;
import android.widget.EditText;
import android.widget.TextView;

/** Compiled checks for DROIDLESS's bounded, approximate text measurement profile. */
public final class TextLayoutContract {
    static class Probe extends TextView {
        boolean fail;
        Probe(Activity context) { super(context); }
        @Override protected void onMeasure(int width, int height) {
            System.gc();
            super.onMeasure(width, height);
            System.gc();
            if (fail) throw new IllegalStateException("measure failure");
        }
    }
    static void check(boolean condition) {
        if (!condition) throw new IllegalStateException("Text layout contract failed");
    }
    static void measure(TextView text, int width) {
        text.measure(View.MeasureSpec.makeMeasureSpec(width, View.MeasureSpec.EXACTLY),
            View.MeasureSpec.makeMeasureSpec(1000, View.MeasureSpec.AT_MOST));
    }
    public static int run(Activity context) {
        Probe text = new Probe(context);
        text.setTextSize(10f);
        text.setText("abcdefghijklmnopqrst");
        check(text.getLayout() == null);
        measure(text, 48);
        Layout old = text.getLayout();
        check(old.getLineCount() == 3 && old.getWidth() == 48);
        check(old.getText().toString().equals("abcdefghijklmnopqrst"));
        check(text.getMeasuredWidth() == 48 && text.getMeasuredHeight() == 54);
        measure(text, 48);
        System.gc();
        check(text.getLayout() == old);
        measure(text, 120);
        check(text.getLayout() != old && text.getLayout().getLineCount() == 1);
        text.setText("ab\n\ncd\n");
        check(text.getLayout() == null && text.isLayoutRequested());
        measure(text, 48);
        check(text.getLayout().getLineCount() == 4 && text.getMeasuredHeight() == 64);
        check(old.getText().toString().equals("abcdefghijklmnopqrst") && old.getLineCount() == 3);
        text.setSingleLine(true);
        check(text.getLayout() == null);
        measure(text, 48);
        check(text.getLayout().getLineCount() == 1);
        text.setSingleLine(false);
        text.setText("abcdefghijklmnopqrst");
        text.setPadding(12, 0, 12, 0);
        measure(text, 48);
        check(text.getLayout().getWidth() == 24 && text.getLayout().getLineCount() == 5);
        text.setTextSize(20f);
        check(text.getLayout() == null);
        measure(text, 48);
        check(text.getLayout().getLineCount() == 10 && text.getMeasuredHeight() == 224);
        text.setPadding(0, 0, 0, 0);
        text.setTextSize(10f);
        text.setText("aa bbbbb c");
        measure(text, 30);
        check(text.getLayout().getLineCount() == 3);
        text.setMaxLines(1);
        measure(text, 30);
        check(text.getLayout().getLineCount() == 3 && text.getMeasuredHeight() == 44);
        text.setMinLines(5);
        measure(text, 30);
        check(text.getMeasuredHeight() == 74);
        text.setText("😀😀😀");
        measure(text, 0);
        check(text.getLayout().getLineCount() == 3 && text.getLayout().getText().length() == 6);
        text.setText("");
        measure(text, 0);
        check(text.getLayout().getLineCount() == 1);
        text.setTextSize(0f);
        text.setText("unbounded\ntext");
        measure(text, 0);
        check(text.getLayout().getLineCount() == 2);
        text.fail = true;
        text.requestLayout();
        try { measure(text, 48); return 0; }
        catch (IllegalStateException expected) { check(text.isLayoutRequested()); }

        EditText edit = new EditText(context);
        edit.setTextSize(10f);
        edit.setText("abcdefgh");
        measure(edit, 48);
        edit.getText().append("i");
        check(edit.getLayout() == null && edit.getText().toString().equals("abcdefghi"));
        measure(edit, 48);
        check(edit.getLayout().getLineCount() == 2);
        return 1;
    }
}
