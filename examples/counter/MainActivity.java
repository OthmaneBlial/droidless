package org.droidless.counter;

import android.app.Activity;
import android.os.Bundle;
import android.view.View;
import android.view.KeyEvent;
import android.widget.TextView;
import android.widget.EditText;

/** Purpose-built DROIDLESS fixture. Ordinary Java compiled with javac and D8. */
public class MainActivity extends Activity {
    private int count;
    private TextView label;
    private static int bias = 3;
    @Override public void onCreate(Bundle state) {
        super.onCreate(state);
        setContentView(R.layout.main);
        label = (TextView) findViewById(R.id.count);
        findViewById(R.id.input).setOnKeyListener(new View.OnKeyListener() {
            public boolean onKey(View view, int code, KeyEvent event) {
                if (event.getAction() != KeyEvent.ACTION_DOWN) return false;
                label.setText("key " + event.getKeyCode());
                return true;
            }
        });
        findViewById(R.id.increment).setOnClickListener(new View.OnClickListener() {
            public void onClick(View view) { label.setText(Integer.toString(++count)); }
        });
        findViewById(R.id.decrement).setOnClickListener(new View.OnClickListener() {
            public void onClick(View view) { label.setText(Integer.toString(--count)); }
        });
        findViewById(R.id.copy).setOnClickListener(new View.OnClickListener() {
            public void onClick(View view) {
                EditText input = (EditText) findViewById(R.id.input);
                label.setText(input.getText().toString());
            }
        });
    }
    public static int sum(int n) { int s=bias; for (int i=0;i<n;i++) s+=i; return s; }
    public static long wideMath(long a, int shift) { return (a << shift) ^ (a >>> (64-shift)); }
    public static double squares(double x, double y) { return x*x+y*y; }
    public static int arrays() { int[] a={2,4,6}; a[1]=9; return a.length+a[0]+a[1]+a[2]; }
    public static int dispatch() { Work w=new Derived(); return w.apply(4); }
    public static int caught() { try { throw new RuntimeException("test"); } catch (RuntimeException e) { return 7; } }
    interface Work { int apply(int x); }
    static class Base { public int apply(int x) { return x+2; } }
    static class Derived extends Base implements Work { public int apply(int x) { return super.apply(x)*3; } }
}
