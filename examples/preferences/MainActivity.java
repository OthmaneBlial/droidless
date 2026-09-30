package org.droidless.preferences;

import android.app.Activity;
import android.content.Intent;
import android.content.SharedPreferences;
import android.os.Bundle;
import android.view.View;
import android.widget.TextView;
import android.widget.EditText;

/** Authored storage/API conformance, not independent notes APK evidence. */
public class MainActivity extends Activity {
    public void onCreate(Bundle state) {
        super.onCreate(state); setTitle("Saved note"); setContentView(R.layout.main);
        ((TextView) findViewById(R.id.status)).setText(contract(this) == 1 ? "Conformance passed" : "BROKEN preferences");
    }
    public void onResume() {
        super.onResume(); refresh();
    }
    private void refresh() {
        ((TextView) findViewById(R.id.note)).setText(getSharedPreferences("note", 0).getString("text", "No saved note"));
    }
    public void edit(View view) { startActivity(new Intent(this, Edit.class)); }
    public void clear(View view) {
        getSharedPreferences("note", 0).edit().remove("text").apply(); refresh();
    }
    public static int contract(Activity context) {
        SharedPreferences p = context.getSharedPreferences("contract", 0);
        if (p != context.getSharedPreferences("contract", 0)) return 0;
        if (p.contains("l")) {
            if (p.getLong("l", 0) != 0x1122334455667788L || p.getFloat("f", 0) != 1.25f || !p.getBoolean("b", false)) return 0;
            float restoredNaN = p.getFloat("nan", 0);
            if (restoredNaN == restoredNaN) return 0;
        }
        if (!p.edit().clear().putString("old", "removed").commit()) return 0;
        SharedPreferences.Editor editor = p.edit();
        editor.putInt("i", 7).putLong("l", 0x1122334455667788L).putFloat("f", 1.25f)
            .putFloat("nan", Float.NaN).putBoolean("b", true).putString("s", "value");
        if (p.contains("i") || p.getInt("i", 21) != 21) return 0;
        // Android clear runs before staged puts, including those preceding clear().
        editor.clear();
        if (!editor.commit() || p.contains("old")) return 0;
        if (p.getInt("i", 0) != 7 || p.getLong("l", 0) != 0x1122334455667788L) return 0;
        if (p.getFloat("f", 0) != 1.25f || !p.getBoolean("b", false)) return 0;
        float nan = p.getFloat("nan", 0);
        if (nan == nan || !"value".equals(p.getString("s", null))) return 0;
        try { p.getString("i", "wrong"); return 0; } catch (ClassCastException expected) {}
        editor.remove("i").putString("s", null).apply();
        if (p.contains("i") || p.contains("s") || p.getString("missing", null) != null) return 0;
        if (context.getPreferences(0).getInt("missing", 42) != 42) return 0;
        if (context.getApplicationContext() != context.getApplicationContext()) return 0;
        SharedPreferences lower = context.getSharedPreferences("case", 0);
        SharedPreferences upper = context.getSharedPreferences("CASE", 0);
        if (lower == upper) return 0;
        if (!lower.edit().putInt("value", 1).commit() || !upper.edit().putInt("value", 2).commit()) return 0;
        if (lower.getInt("value", 0) != 1 || upper.getInt("value", 0) != 2) return 0;
        return 1;
    }
    public static class Edit extends Activity {
        public void onCreate(Bundle state) {
            super.onCreate(state); setTitle("Edit saved note"); setContentView(R.layout.edit);
            ((EditText) findViewById(R.id.input)).setText(getSharedPreferences("note", 0).getString("text", ""));
        }
        public void save(View view) {
            String text = ((EditText) findViewById(R.id.input)).getText().toString();
            if (getSharedPreferences("note", 0).edit().putString("text", text).commit()) finish();
            else setTitle("Save failed");
        }
        public void cancel(View view) { finish(); }
    }
}
