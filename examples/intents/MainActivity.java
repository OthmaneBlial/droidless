package org.droidless.intents;

import android.app.Activity;
import android.content.Intent;
import android.os.Bundle;
import android.view.View;
import android.widget.TextView;

/** Authored conformance fixture, never third-party compatibility evidence. */
public class MainActivity extends Activity {
    public static String events = "";
    public static int finishing;
    public static boolean creating;
    private static MainActivity home;
    private int resumes;
    public void onCreate(Bundle state) {
        creating=true;
        super.onCreate(state);
        home = this;
        events += "home:create;";
        setTitle("Home");
        setContentView(R.layout.main);
        FragmentProbe.install(this);
        creating=false;
    }
    public void onStart() { super.onStart(); events += "home:start;"; }
    public void onResume() {
        super.onResume(); events += "home:resume;";
        ((TextView) findViewById(R.id.status)).setText("Home resume " + (++resumes));
    }
    public void onPause() { super.onPause(); events += "home:pause;"; }
    public void onStop() { super.onStop(); events += "home:stop;"; }
    public void onRestart() { super.onRestart(); events += "home:restart;"; }
    public void onDestroy() {
        super.onDestroy(); events += "home:destroy;";
        if (isFinishing()) finishing++;
    }
    public void open(View view) {
        Bundle extras = new Bundle();
        extras.putString("message", "Original extras");
        extras.putInt("int", 42);
        extras.putLong("long", 0x1122334455667788L);
        extras.putFloat("float", 1.25f);
        extras.putDouble("double", 3.5);
        extras.putBoolean("bool", true);
        Intent intent = new Intent(this, Detail.class).putExtras(extras);
        startActivity(intent);
        intent.putExtra("message", "Changed after start");
        extras.putInt("int", 99);
        events += "home:callback-return;";
    }
    public void guarded(View view) {
        startActivity(new Intent().setClassName(getPackageName(), ".MainActivity$Guarded"));
    }
    public static String eventLog() { return events; }
    public static int finishCount() { return finishing; }
    public static int bundleValues() {
        Bundle original = new Bundle();
        original.putString("s", "value");
        original.putInt("i", 7);
        original.putLong("l", 0x1122334455667788L);
        original.putFloat("f", 1.25f);
        original.putDouble("d", 3.5);
        original.putBoolean("b", true);
        original.putString("null", null);
        Bundle copy = new Bundle(original);
        original.clear();
        if (!original.isEmpty() || copy.size() != 7 || !copy.containsKey("null")) return 0;
        if (!"value".equals(copy.getString("s")) || copy.getInt("i") != 7) return 0;
        if (copy.getLong("l") != 0x1122334455667788L || copy.getFloat("f") != 1.25f) return 0;
        if (copy.getDouble("d") != 3.5 || !copy.getBoolean("b")) return 0;
        if (copy.getString("null") != null || copy.getInt("s", 21) != 21) return 0;
        if (copy.getDouble("missing", 8.5) != 8.5 || copy.getLong("missing") != 0) return 0;
        copy.remove("s");
        return copy.containsKey("s") ? 0 : 1;
    }
    public static class Detail extends Activity {
        public void onCreate(Bundle state) {
            super.onCreate(state); events += "detail:create;";
            setTitle("Detail"); setContentView(R.layout.detail);
            Intent intent = getIntent();
            Bundle extras = intent.getExtras();
            boolean valid = extras != null && extras.getInt("int") == 42
                && intent.getLongExtra("long", 0) == 0x1122334455667788L
                && intent.getFloatExtra("float", 0) == 1.25f
                && intent.getDoubleExtra("double", 0) == 3.5
                && intent.getBooleanExtra("bool", false)
                && intent.getIntExtra("missing", 17) == 17;
            extras.putString("message", "Changed extras copy");
            ((TextView) findViewById(R.id.status)).setText(valid ? intent.getStringExtra("message") : "BROKEN extras");
        }
        public void onStart() { super.onStart(); events += "detail:start;"; }
        public void onResume() { super.onResume(); events += "detail:resume;"; }
        public void onPause() { super.onPause(); events += "detail:pause;"; }
        public void onStop() { super.onStop(); events += "detail:stop;"; }
        public void onDestroy() {
            super.onDestroy(); events += "detail:destroy;";
            if (isFinishing()) finishing++;
        }
        public void done(View view) { finish(); }
        public void finishHome(View view) { home.finish(); }
    }
    public static class Guarded extends Activity {
        private boolean blocked;
        public void onCreate(Bundle state) {
            super.onCreate(state); setTitle("Guarded"); setContentView(R.layout.detail);
            ((TextView) findViewById(R.id.status)).setText("First Back is intercepted");
        }
        public void onBackPressed() {
            if (!blocked) {
                blocked = true;
                ((TextView) findViewById(R.id.status)).setText("Back intercepted");
            } else super.onBackPressed();
        }
        public void done(View view) { finish(); }
        public void finishHome(View view) { home.finish(); }
    }
}
