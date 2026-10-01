package org.droidless.counter;

import android.app.Activity;
import android.content.Intent;
import android.os.Bundle;
import android.view.Menu;
import android.view.MenuItem;
import android.widget.TextView;

/** Authored lifecycle/menu callbacks for the native host contract. */
public class MenuActivity extends Activity {
    static int mode, creates, prepares, listeners, selections;
    private TextView label;
    public static void open(Activity activity) { activity.startActivity(new Intent(activity, MenuActivity.class)); }
    public static void configure(int value) { mode = value; }
    public static String counts() { return creates + ":" + prepares + ":" + listeners + ":" + selections; }
    @Override public void onCreate(Bundle state) {
        super.onCreate(state); label = new TextView(this); label.setText("Menu ready"); setContentView(label);
    }
    @Override public boolean onCreateOptionsMenu(Menu menu) {
        creates++; System.gc();
        if (mode == 4) { mode = 0; throw new IllegalStateException("create menu failed"); }
        String[] titles = {"Handled", "Fallback", "Disabled", "Hidden", "Checked", "Finish", "Throw", "Invalidate", "Start"};
        for (int id = 1; id <= titles.length; id++) {
            MenuItem item = menu.add(0, id, id, titles[id-1]);
            if (id == 3) item.setEnabled(false);
            if (id == 4) item.setVisible(false);
            if (id == 5) item.setCheckable(true).setChecked(true);
            if (id == 1 || id == 2 || id == 3 || id == 4 || id == 7 || id == 8) {
                item.setOnMenuItemClickListener(new MenuItem.OnMenuItemClickListener() {
                    public boolean onMenuItemClick(MenuItem selected) {
                        listeners++; System.gc();
                        int value = selected.getItemId();
                        if (value == 7) throw new IllegalStateException("menu listener failed");
                        if (value == 8) invalidateOptionsMenu();
                        label.setText("Listener " + value);
                        return value == 1;
                    }
                });
            }
        }
        if (mode == 1) { mode = 0; return false; }
        if (mode == 3) { mode = 0; invalidateOptionsMenu(); }
        if (mode == 5) { mode = 0; finish(); }
        return true;
    }
    @Override public boolean onPrepareOptionsMenu(Menu menu) {
        prepares++; System.gc();
        if (mode == 2) { mode = 0; return false; }
        if (mode == 6) { mode = 0; throw new IllegalStateException("prepare menu failed"); }
        menu.findItem(1).setTitle("Handled " + prepares);
        return true;
    }
    @Override public boolean onOptionsItemSelected(MenuItem item) {
        selections++; System.gc(); label.setText("Activity " + item.getItemId());
        if (item.getItemId() == 6) finish();
        if (item.getItemId() == 9) open(this);
        return false; // Acting does not require returning true.
    }
}
