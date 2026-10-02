package org.droidless.images;

import android.app.Activity;
import android.text.Editable;
import android.text.TextWatcher;
import android.widget.EditText;

/** Real guest callbacks, UTF-16 deltas, mutable buffers, GC and failure recovery. */
public final class TextWatcherContract {
    static String events="";
    static Editor editor;
    static int fail;
    static boolean append;
    static void check(boolean value) {
        if (!value) throw new IllegalStateException("text watcher contract: "+events);
    }
    static final class Editor extends EditText {
        Editor() { super(new Activity()); }
        @Override protected void onTextChanged(CharSequence s,int start,int before,int count) {
            super.onTextChanged(s,start,before,count);
            events += "H"; System.gc();
        }
    }
    static final class Watcher implements TextWatcher {
        public void beforeTextChanged(CharSequence s,int start,int count,int after) {
            events += "B"+start+":"+count+":"+after+";"; System.gc();
            if (fail==1) throw new IllegalStateException("before failure");
        }
        public void onTextChanged(CharSequence s,int start,int before,int count) {
            events += "O"; System.gc();
            if (fail==2) throw new IllegalStateException("on failure");
        }
        public void afterTextChanged(Editable s) {
            events += "A"; System.gc();
            check(s==editor.getText());
            if (append) { append=false; s.append("!"); }
            if (fail==3) throw new IllegalStateException("after failure");
        }
    }
    public static EditText run() {
        editor=new Editor(); editor.setText("old");
        Watcher watcher=new Watcher(); editor.addTextChangedListener(watcher);
        Editable old=editor.getText(); events=""; editor.setText("A\uD83D\uDE80");
        check(events.equals("B0:3:3;OHA") && editor.getText()!=old);
        old.append(" detached"); check(editor.getText().toString().equals("A\uD83D\uDE80"));
        Editable current=editor.getText(); events=""; append=true; current.append("x");
        check(events.equals("B3:0:1;OHAB4:0:1;OHA"));
        check(editor.getText()==current && current.toString().equals("A\uD83D\uDE80x!"));
        for (int stage=1;stage<=3;stage++) {
            fail=stage; events="";
            try { editor.setText("changed"); throw new AssertionError("fault skipped"); }
            catch(IllegalStateException expected) {}
            check(editor.getText().toString().equals(stage==1 ? "A\uD83D\uDE80x!" : "changed"));
        }
        fail=0; editor.removeTextChangedListener(watcher); events=""; editor.setText("reset");
        check(events.equals("H")); editor.addTextChangedListener(watcher); events="";
        return editor;
    }
    public static void hostCheck() {
        check(events.equals("B0:5:4;OHA") && editor.getText().toString().equals("host"));
        editor=null;
    }
}
