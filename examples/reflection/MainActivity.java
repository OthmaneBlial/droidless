package org.droidless.reflection;
import android.app.Activity;
import android.os.Bundle;
import android.widget.TextView;

public class MainActivity extends Activity {
    public void onCreate(Bundle state) {
        super.onCreate(state);
        TextView label = new TextView(this);
        try { label.setText(ReflectionContract.run() == 1 && PrimitiveContract.run() == 1 ? "Reflection passed" : "BROKEN reflection"); }
        catch (Exception failure) { throw new RuntimeException("Reflection contract failed"); }
        setContentView(label);
    }
}
