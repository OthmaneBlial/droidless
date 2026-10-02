package org.droidless.images;

import android.app.Activity;
import android.app.Dialog;
import android.content.Context;
import android.content.DialogInterface;
import android.os.Bundle;
import android.view.View;
import android.view.ViewGroup;
import android.widget.Button;
import android.widget.EditText;
import android.widget.LinearLayout;
import android.widget.TextView;

public final class DialogSurfaceContract {
    static Probe dialog, upper;
    static Button background;
    static int creates, starts, stops, clicks, backgroundClicks, attached, detached;
    static String events = "";
    static boolean failCreate, failStart, failStop;
    static final class Probe extends Dialog {
        final String label;
        Probe(Context context, String label) { super(context); this.label = label; }
        @Override protected void onCreate(Bundle saved) {
            creates++;
            if (failCreate) throw new IllegalStateException("dialog create failure");
            super.onCreate(saved);
            LinearLayout layout = new LinearLayout(getContext());
            layout.setOrientation(LinearLayout.VERTICAL);
            TextView title = new TextView(getContext()); title.setText(label); layout.addView(title);
            EditText editor = new EditText(getContext()); editor.setText("dialog input"); layout.addView(editor);
            Button button = new Button(getContext()); button.setText("Confirm dialog");
            button.setOnClickListener(new View.OnClickListener() {
                public void onClick(View view) { clicks++; dismiss(); System.gc(); }
            });
            layout.addView(button);
            setContentView(layout);
            getWindow().setLayout(240, 160);
            setTitle(label);
            setOnShowListener(new DialogInterface.OnShowListener() {
                public void onShow(DialogInterface owner) { events += "S"; System.gc(); }
            });
            setOnCancelListener(new DialogInterface.OnCancelListener() {
                public void onCancel(DialogInterface owner) { events += "C"; System.gc(); }
            });
            setOnDismissListener(new DialogInterface.OnDismissListener() {
                public void onDismiss(DialogInterface owner) { events += "D"; System.gc(); }
            });
        }
        @Override public void onAttachedToWindow() { attached++; System.gc(); super.onAttachedToWindow(); }
        @Override public void onDetachedFromWindow() { detached++; System.gc(); super.onDetachedFromWindow(); }
        @Override protected void onStart() {
            starts++; System.gc();
            if (failStart) throw new IllegalStateException("dialog start failure");
            super.onStart();
        }
        @Override protected void onStop() {
            stops++; System.gc();
            if (failStop) throw new IllegalStateException("dialog stop failure");
            super.onStop();
        }
    }
    public static Dialog begin(Activity activity) {
        background = new Button(activity); background.setText("Background action");
        background.setOnClickListener(new View.OnClickListener() {
            public void onClick(View view) { backgroundClicks++; }
        });
        ((ViewGroup)activity.getWindow().getDecorView()).addView(background);
        dialog = new Probe(activity, "Separate dialog"); dialog.show(); return dialog;
    }
    public static View background() { return background; }
    public static void size(int width, int height) { dialog.getWindow().setLayout(width, height); }
    public static void hide() { dialog.hide(); }
    public static void show() { dialog.show(); }
    public static void cancelable(boolean value) { dialog.setCancelable(value); }
    public static void outside(boolean value) { dialog.setCanceledOnTouchOutside(value); }
    public static Dialog push() { upper = new Probe(dialog.getContext(), "Upper dialog"); upper.show(); return upper; }
    public static void dismissUpper() { upper.dismiss(); upper = null; }
    public static boolean showing() { return dialog.isShowing(); }
    public static void dismiss() { dialog.dismiss(); }
    public static void drop() { dialog = null; upper = null; background = null; }
    public static String attachments() { return attached + ":" + detached; }
    public static String state() { return creates + ":" + starts + ":" + stops + ":" + clicks + ":" + backgroundClicks + ":" + events; }
    public static Dialog failed(Activity activity, int phase) {
        dialog = new Probe(activity, "Recovery dialog");
        failCreate = phase == 1; failStart = phase == 2; dialog.show(); return dialog;
    }
    public static void recover() { failCreate = false; failStart = false; failStop = false; dialog.show(); }
    public static void failDismiss() { failStop = true; dialog.dismiss(); }
}
