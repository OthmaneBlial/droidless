package org.droidless.counter;

import android.app.Activity;
import android.text.Layout;
import android.view.View;
import android.view.ViewGroup;
import android.widget.EditText;
import android.widget.FrameLayout;
import android.widget.LinearLayout;
import android.widget.TextView;

/** Compiled checks for DROIDLESS's bounded, approximate text measurement profile. */
public final class TextLayoutContract {
    private static FrameLayout retained;
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
    static class Row extends LinearLayout {
        int layouts;
        boolean failLayout;
        Row(Activity context) { super(context); }
        @Override protected void onMeasure(int width, int height) {
            System.gc();
            super.onMeasure(width, height);
            check(((TextView)getChildAt(0)).getLayout() != null);
            System.gc();
        }
        @Override protected void onLayout(boolean changed, int left, int top, int right, int bottom) {
            layouts++;
            System.gc();
            if (failLayout) throw new IllegalStateException("layout failure");
            super.onLayout(changed, left, top, right, bottom);
        }
    }
    static class MutatingProbe extends Probe {
        MutatingProbe(Activity context) { super(context); }
        @Override protected void onMeasure(int width, int height) {
            super.onMeasure(width, height);
            ((ViewGroup)getParent()).removeView(this);
        }
    }
    static void check(boolean condition) {
        if (!condition) throw new IllegalStateException("Text layout contract failed");
    }
    static void measure(View text, int width) {
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

        android.support.v7.widget.RecyclerView list = new android.support.v7.widget.RecyclerView(context);
        measure(list, 150);
        measure(list, 150);
        check(list.clears == 2);

        Row row = new Row(context);
        check(row.getOrientation() == LinearLayout.HORIZONTAL);
        row.setOrientation(LinearLayout.VERTICAL);
        check(row.getOrientation() == LinearLayout.VERTICAL);
        row.setOrientation(LinearLayout.HORIZONTAL);
        row.setPadding(6, 2, 6, 2);
        Probe message = new Probe(context);
        message.setTextSize(10f);
        message.setText("abcdefghijklmnopqrst");
        LinearLayout.LayoutParams weighted = new LinearLayout.LayoutParams(0, -2, 1f);
        weighted.setMargins(2, 0, 4, 0);
        row.addView(message, weighted);
        Probe button = new Probe(context);
        row.addView(button, new LinearLayout.LayoutParams(42, 30));
        Probe hidden = new Probe(context);
        hidden.fail = true;
        hidden.setVisibility(View.GONE);
        row.addView(hidden, new LinearLayout.LayoutParams(20, 20));
        FrameLayout frame = new FrameLayout(context);
        frame.setPadding(6, 3, 6, 3);
        frame.addView(row, new FrameLayout.LayoutParams(-1, -2));
        measure(frame, 162);
        check(message.getLayout().getWidth() == 90 && message.getLayout().getLineCount() == 2);
        check(button.getMeasuredWidth() == 42 && row.getMeasuredHeight() == 48);
        check(frame.getMeasuredHeight() == 54 && row.getMeasuredWidth() == 150);
        frame.measure(View.MeasureSpec.makeMeasureSpec(162, View.MeasureSpec.AT_MOST),
            View.MeasureSpec.makeMeasureSpec(1000, View.MeasureSpec.AT_MOST));
        check(frame.getMeasuredWidth() == 162 && message.getLayout().getWidth() == 90);
        frame.measure(View.MeasureSpec.makeMeasureSpec(0, View.MeasureSpec.UNSPECIFIED),
            View.MeasureSpec.makeMeasureSpec(0, View.MeasureSpec.UNSPECIFIED));
        check(frame.getMeasuredWidth() == 216 && message.getLayout().getWidth() == 144);
        measure(frame, 162);
        message.setText("abc");
        check(row.isLayoutRequested() && frame.isLayoutRequested());
        message.fail = true;
        try { measure(frame, 162); return 0; }
        catch (IllegalStateException expected) { check(frame.isLayoutRequested()); }
        message.fail = false;
        measure(frame, 162);
        check(message.getLayout().getLineCount() == 1 && frame.getMeasuredHeight() == 54);
        retained = frame;
        return 1;
    }
    public static View root() { return retained; }
    public static int layoutCount() { return ((Row)retained.getChildAt(0)).layouts; }
    public static void remeasure() { measure(retained, 162); }
    public static void failLayout() {
        ((Row)retained.getChildAt(0)).failLayout = true;
        remeasure();
    }
    public static void recoverLayout() { ((Row)retained.getChildAt(0)).failLayout = false; }
    public static void release() { retained = null; }
    public static void mutate(Activity context) {
        LinearLayout row = new LinearLayout(context);
        row.addView(new MutatingProbe(context), new LinearLayout.LayoutParams(100, 50));
        measure(row, 150);
    }
}
