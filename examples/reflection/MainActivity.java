package org.droidless.reflection;
import android.app.Activity;
import android.os.Bundle;
import android.os.Build;
import android.widget.TextView;

public class MainActivity extends Activity {
    static int aliasInitializations;
    public static class ApiAlias extends Build.VERSION {
        static { aliasInitializations++; }
    }
    public static int readSdk() { return Build.VERSION.SDK_INT; }
    public static int readAliasSdk() { return ApiAlias.SDK_INT; }
    public void onCreate(Bundle state) {
        super.onCreate(state);
        TextView label = new TextView(this);
        try {
            int sdk=readSdk(); System.gc();
            if (sdk!=21 || readSdk()!=sdk || readAliasSdk()!=sdk || aliasInitializations!=0 || sdk<14 || sdk>=22
                    || Class.forName("android.os.Build$VERSION")!=Build.VERSION.class)
                throw new IllegalStateException("Virtual API profile changed");
            label.setText(ReflectionContract.run() == 1 && PrimitiveContract.run() == 1 ? "Reflection passed" : "BROKEN reflection");
        }
        catch (Exception failure) { throw new RuntimeException("Reflection contract failed"); }
        setContentView(label);
    }
}
