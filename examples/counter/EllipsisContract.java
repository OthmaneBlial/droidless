package org.droidless.counter;

import android.app.Activity;
import android.text.Layout;
import android.text.TextUtils.TruncateAt;
import android.view.LayoutInflater;
import android.view.View;
import android.widget.EditText;
import android.widget.TextView;

/** Checks the bounded scalar-width profile, UTF-16 metadata and immutable snapshots. */
public final class EllipsisContract {
    static final class Probe extends TextView {
        Probe(Activity context) { super(context); }
        @Override protected void onMeasure(int width, int height) {
            System.gc();
            super.onMeasure(width, height);
            System.gc();
        }
    }
    static void check(boolean value) {
        if (!value) throw new IllegalStateException("Ellipsis contract failed");
    }
    static Layout measure(TextView text, int width) {
        text.measure(View.MeasureSpec.makeMeasureSpec(width, View.MeasureSpec.EXACTLY),
            View.MeasureSpec.makeMeasureSpec(1000, View.MeasureSpec.AT_MOST));
        return text.getLayout();
    }
    static void ellipsis(Layout layout, int line, int start, int count) {
        check(layout.getEllipsisStart(line) == start && layout.getEllipsisCount(line) == count);
    }
    public static TextView run(Activity context) {
        Probe text = new Probe(context);
        text.setTextSize(10f);
        text.setSingleLine(true);
        text.setText("abcdefghij");
        check(text.getEllipsize() == null);
        ellipsis(measure(text, 30), 0, 0, 0);
        text.setEllipsize(TruncateAt.END);
        check(text.getLayout() == null && text.isLayoutRequested());
        Layout old = measure(text, 30);
        ellipsis(old, 0, 4, 6);
        check(old.getText().toString().equals("abcd\u2026\ufeff\ufeff\ufeff\ufeff\ufeff"));
        check(text.getText().toString().equals("abcdefghij"));
        text.setEllipsize(TruncateAt.END);
        check(text.getLayout() == old && measure(text, 30) == old);
        text.setEllipsize(TruncateAt.START);
        ellipsis(measure(text, 30), 0, 0, 6);
        check(text.getLayout().getText().toString().equals("\u2026\ufeff\ufeff\ufeff\ufeff\ufeffghij"));
        text.setEllipsize(TruncateAt.MIDDLE);
        ellipsis(measure(text, 30), 0, 2, 6);
        check(text.getLayout().getText().toString().equals("ab\u2026\ufeff\ufeff\ufeff\ufeff\ufeffij"));
        System.gc();
        ellipsis(old, 0, 4, 6);
        text.setEllipsize(null);
        check(text.getLayout() == null && text.getEllipsize() == null);
        check(measure(text, 30).getText().toString().equals("abcdefghij"));
        text.setEllipsize(TruncateAt.MARQUEE);
        ellipsis(measure(text, 30), 0, 0, 0);
        text.setEllipsize(TruncateAt.END);
        ellipsis(measure(text, 60), 0, 0, 0);
        text.setText("\ud83d\ude00\ud83d\ude00\ud83d\ude00x");
        Layout unicode = measure(text, 18);
        ellipsis(unicode, 0, 4, 3);
        check(unicode.getText().length() == 7);
        check(unicode.getText().toString().equals("\ud83d\ude00\ud83d\ude00\u2026\ufeff\ufeff"));
        text.setEllipsize(TruncateAt.START);
        ellipsis(measure(text, 18), 0, 0, 4);
        text.setEllipsize(TruncateAt.MIDDLE);
        ellipsis(measure(text, 18), 0, 2, 4);
        text.setEllipsize(TruncateAt.END);
        ellipsis(measure(text, 0), 0, 0, 7);
        text.setText("");
        ellipsis(measure(text, 0), 0, 0, 0);
        text.setTextSize(0f);
        text.setText("zero width");
        ellipsis(measure(text, 0), 0, 0, 0);
        text.setTextSize(10f);
        text.setSingleLine(false);
        text.setMaxLines(2);
        text.setText("abcdefghijklmno");
        Layout multi = measure(text, 30);
        check(multi.getLineCount() == 2);
        ellipsis(multi, 0, 0, 0);
        ellipsis(multi, 1, 4, 1);
        check(multi.getText().toString().equals("abcdefghi\u2026klmno"));
        text.setEllipsize(TruncateAt.START);
        ellipsis(measure(text, 30), 1, 0, 0);
        text.setEllipsize(TruncateAt.MIDDLE);
        ellipsis(measure(text, 30), 1, 0, 0);
        text.setEllipsize(null);
        check(measure(text, 30).getLineCount() == 3);
        try { text.getLayout().getEllipsisCount(-1); check(false); }
        catch (IndexOutOfBoundsException expected) { }
        try { text.getLayout().getEllipsisStart(3); check(false); }
        catch (IndexOutOfBoundsException expected) { }
        EditText edit = new EditText(context);
        edit.setTextSize(10f);
        edit.setSingleLine(true);
        edit.setText("abcdefghij");
        edit.setEllipsize(TruncateAt.END);
        ellipsis(measure(edit, 30), 0, 0, 0);
        check(edit.getText().toString().equals("abcdefghij"));
        edit.setKeyListener(null);
        check(edit.getLayout() == null);
        ellipsis(measure(edit, 30), 0, 4, 6);
        TextView xml = (TextView) LayoutInflater.from(context).inflate(R.layout.ellipsis, null);
        check(xml.getEllipsize() == TruncateAt.END);
        check(measure(xml, 30).getLineCount() == 1);
        ellipsis(xml.getLayout(), 0, 4, 6);
        check(xml.getMeasuredHeight() == 44);
        return xml;
    }
}
