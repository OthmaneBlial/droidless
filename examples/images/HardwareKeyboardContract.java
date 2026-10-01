package org.droidless.images;

import android.app.Activity;
import android.content.Context;
import android.content.ContextWrapper;
import android.view.View;
import android.view.inputmethod.InputMethodManager;
import android.widget.EditText;

/** Hardware text entry is available; there is no served software input method. */
public final class HardwareKeyboardContract {
    static void check(boolean condition) {
        if (!condition) throw new IllegalStateException("hardware keyboard service contract");
    }
    public static View run(Activity context) {
        InputMethodManager service = (InputMethodManager) context.getSystemService(Context.INPUT_METHOD_SERVICE);
        check(service != null); System.gc();
        check(service == new ContextWrapper(context).getSystemService(Context.INPUT_METHOD_SERVICE));
        EditText editor = new EditText(context);
        check(!service.showSoftInput(editor, InputMethodManager.SHOW_IMPLICIT));
        check(!service.showSoftInput(null, 0) && !service.hideSoftInputFromWindow(null, 0));
        context.setContentView(editor); editor.setText("Hardware text");
        check(editor.requestFocus() && editor.getWindowToken() != null);
        System.gc();
        check(!service.showSoftInput(editor, InputMethodManager.SHOW_FORCED));
        check(!service.hideSoftInputFromWindow(editor.getWindowToken(), InputMethodManager.HIDE_NOT_ALWAYS));
        check(editor.isFocused() && "Hardware text".equals(editor.getText().toString()));
        editor.clearFocus();
        check(!service.showSoftInput(editor, 0));
        return editor;
    }
}
