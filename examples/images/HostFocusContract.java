package org.droidless.images;

import android.app.Activity;
import android.view.View;
import android.widget.EditText;
import android.widget.LinearLayout;

/** Host focus executes real editor callbacks, including GC and failures. */
public final class HostFocusContract {
    static LinearLayout root;
    static EditText first,second;
    static int gains,losses;
    static boolean fail;
    static EditText editor(Activity context) {
        EditText editor=new EditText(context);
        editor.setOnFocusChangeListener(new View.OnFocusChangeListener() {
            public void onFocusChange(View view,boolean focused) {
                System.gc();
                if(view.isFocused()!=focused) throw new AssertionError("host focus state");
                if(focused) gains++; else losses++;
                if(fail && focused) throw new IllegalStateException("host focus failure");
            }
        });
        return editor;
    }
    public static EditText prepare(Activity context) {
        gains=losses=0; fail=false; root=new LinearLayout(context);
        first=editor(context); second=editor(context); root.addView(first); root.addView(second);
        return first;
    }
    public static EditText second() { return second; }
    public static void state(int id,int expectedGains,int expectedLosses) {
        EditText expected=id==1?first:second;
        if(root.findFocus()!=expected || !expected.isFocused() || gains!=expectedGains || losses!=expectedLosses)
            throw new AssertionError("host focus ownership/counts");
    }
    public static void fail(boolean value) { fail=value; }
    public static void release() { root=null; first=second=null; }
}
