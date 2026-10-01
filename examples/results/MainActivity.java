package org.droidless.results;

import android.app.Activity;
import android.content.Intent;
import android.net.Uri;
import android.os.Bundle;
import android.os.IBinder;
import android.view.View;
import android.view.ViewGroup;
import android.widget.Button;
import android.widget.LinearLayout;
import android.widget.TextView;

/** Authored DEX contracts: real results and host choices, not a third-party workflow. */
public class MainActivity extends Activity {
    static { System.gc(); }
    static MainActivity home;
    static Child child;
    static Intent received, failed;
    static int calls;
    static boolean throwResult, finishOnResult;
    static String events = "";
    boolean resumed;
    public void onCreate(Bundle state) {
        super.onCreate(state); home = this;
        if (!Intent.ACTION_MAIN.equals(getIntent().getAction()) || getIntent().getComponent() == null)
            throw new IllegalStateException("launcher Intent metadata/GC");
        checkIntent(); setContentView(R.layout.main);
        View status = findViewById(R.id.status);
        if (status.getWindowToken() != null || status.isAttachedToWindow())
            throw new IllegalStateException("View attached during onCreate");
    }
    public void onResume() { super.onResume(); resumed = true; events += "resume;"; }
    public void onPause() { super.onPause(); resumed = false; events += "pause;"; }
    public void onStart() { super.onStart(); events += "start;"; }
    public void open(View view) { startActivityForResult(new Intent(this, Child.class), 7); }
    public void withoutResult(View view) { startActivityForResult(new Intent(this, Child.class), -1); }
    public void pick(View view) { startActivityForResult(new Intent(Intent.ACTION_OPEN_DOCUMENT_TREE), 404); }
    public void onActivityResult(int request, int code, Intent data) {
        super.onActivityResult(request, code, data); System.gc();
        if (resumed) throw new IllegalStateException("result delivered while resumed");
        events += "result;";
        if (throwResult) { failed = data; throw new IllegalStateException("result callback failed"); }
        calls++; received = data;
        String value = data == null ? "none" : request == 404 ? data.getData().toString()
            : data.getStringExtra("value") + ":" + data.getType();
        ((TextView)findViewById(R.id.status)).setText("Result " + request + ":" + code + ":" + value);
        if (finishOnResult) finish();
    }
    static void checkIntent() {
        String action = new StringBuilder("org.droidless.results.TEST").toString();
        String interned = "org.droidless.results.TEST";
        String type = "image/png";
        Uri uri = Uri.parse("content://test/images/42");
        Intent intent = new Intent(action, uri);
        if (intent.getAction() != interned || intent.getData() != uri || intent.getType() != null)
            throw new IllegalStateException("Intent constructor state");
        if (intent.setType(type) != intent || intent.getData() != null || intent.getType() != type)
            throw new IllegalStateException("setType must clear data");
        intent.setData(uri);
        if (intent.getType() != null || intent.getData() != uri)
            throw new IllegalStateException("setData must clear type");
        intent.setDataAndType(uri, type).putExtra("copy", "before");
        Intent copy = new Intent(intent);
        intent.setAction(null).setDataAndType(null, null).putExtra("copy", "after"); System.gc();
        if (copy.getAction() != interned || copy.getType() != type || copy.getData() != uri
                || !"before".equals(copy.getStringExtra("copy")))
            throw new IllegalStateException("Intent copy or retention");
        TracedIntent traced = new TracedIntent(action, uri);
        if (TracedIntent.calls != 1 || traced.getAction() != interned || traced.getData() != uri)
            throw new IllegalStateException("Intent constructor did not dispatch setAction");
    }
    public static class TracedIntent extends Intent {
        static int calls;
        TracedIntent(String action, Uri uri) { super(action, uri); }
        public Intent setAction(String action) { calls++; System.gc(); return super.setAction(action); }
    }
    static void button(Activity activity, LinearLayout layout, String text, final Runnable action) {
        Button button = new Button(activity); button.setText(text);
        button.setLayoutParams(new LinearLayout.LayoutParams(-1, 52));
        button.setOnClickListener(new View.OnClickListener() { public void onClick(View view) { action.run(); } });
        layout.addView(button);
    }
    public static class Child extends Activity {
        public void onCreate(Bundle state) {
            super.onCreate(state); child = this; setTitle("Child result");
            LinearLayout layout = new LinearLayout(this); layout.setOrientation(1); setContentView(layout);
            button(this, layout, "Return result", new Runnable() { public void run() { send(); } });
            button(this, layout, "Cancel result", new Runnable() { public void run() { setResult(RESULT_CANCELED); finish(); } });
            button(this, layout, "Open overlay", new Runnable() { public void run() { startActivity(new Intent(Child.this, Overlay.class)); } });
            button(this, layout, "Finish caller", new Runnable() { public void run() { home.finish(); } });
        }
        void send() {
            Intent result = new Intent("org.droidless.results.RETURN").setType("image/png").putExtra("value", "before finish");
            setResult(RESULT_OK, result); result.putExtra("value", "at finish"); System.gc();
            finish(); result.putExtra("value", "too late"); setResult(17, null); finish(); System.gc();
        }
        public void onDestroy() { super.onDestroy(); if (child == this) child = null; }
    }
    public static class Overlay extends Activity {
        public void onCreate(Bundle state) {
            super.onCreate(state); setTitle("Overlay");
            LinearLayout layout = new LinearLayout(this); layout.setOrientation(1); setContentView(layout);
            button(this, layout, "Finish stopped child", new Runnable() { public void run() { child.send(); } });
        }
    }
    public static int resultCount() { return calls; }
    public static String eventLog() { return events; }
    public static Intent resultData() { return received; }
    public static void dropResult() { received = null; }
    public static void failResult() { throwResult = true; }
    public static void finishAfterResult() { finishOnResult = true; }
    public static Intent takeFailedResult() { Intent result = failed; failed = null; return result; }
    public static Child childObject() { return child; }
    public static IBinder windowToken() { return home.findViewById(R.id.status).getWindowToken(); }
    public static void checkAttachment() {
        View status = home.findViewById(R.id.status);
        IBinder token = status.getWindowToken();
        if (token == null || !status.isAttachedToWindow()) throw new IllegalStateException("missing window token");
        View view = new View(home);
        if (view.getWindowToken() != null) throw new IllegalStateException("unattached View has token");
        ViewGroup root = (ViewGroup)status.getParent(); root.addView(view); System.gc();
        if (view.getWindowToken() != token || !view.isAttachedToWindow()) throw new IllegalStateException("attached child token");
        root.removeView(view);
        if (view.getWindowToken() != null || view.isAttachedToWindow()) throw new IllegalStateException("detached child token");
    }
}
