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
    private static int divide(int n) { return 42/n; }
    public static int faults(int which) {
        try {
            switch (which) {
                case 0: return divide(bias-3);
                case 1: return (int) (42L/(bias-3));
                case 2: return new int[bias-4].length;
                case 3: return (new int[1])[2];
                case 4: return (new int[1])[-1];
                case 5: return ((String) null).length();
                case 6: return ((MainActivity) null).count;
                case 7: throw (RuntimeException) null;
                case 8: Object obj=new Base(); return ((String) obj).length();
                case 9: return Integer.parseInt("invalid");
                case 10: return (int) Double.parseDouble("invalid");
                case 11: return "a".charAt(2);
                case 12: return "abc".substring(2,1).length();
                case 13: Object[] values=new String[1]; values[0]=new Object(); return 0;
                case 14: return ((int[]) null)[-1];
                case 15: return Integer.parseInt(null);
                case 16: throw new NumberFormatException("authored");
                default: return 0;
            }
        } catch (ArithmeticException e) { return 1; }
        catch (NullPointerException e) { return 2; }
        catch (NegativeArraySizeException e) { return 3; }
        catch (ArrayIndexOutOfBoundsException e) { return 4; }
        catch (ClassCastException e) { return 5; }
        catch (NumberFormatException e) { return e.getMessage().length()>0 ? 6 : -6; }
        catch (StringIndexOutOfBoundsException e) { return 7; }
        catch (ArrayStoreException e) { return 8; }
    }
    private static int finallyRuns;
    public static int finallyFault() {
        try { try { return divide(0); } finally { finallyRuns++; } }
        catch (Exception e) { return finallyRuns; }
    }
    public static int inheritedTypes() {
        RootWork w=new Derived(); Object array=new Derived[1];
        return w instanceof RootWork && array instanceof Base[] && array instanceof Object[] ? w.apply(4) : -1;
    }
    interface RootWork { int apply(int x); }
    interface Work extends RootWork {}
    static class Base { public int apply(int x) { return x+2; } }
    static class Derived extends Base implements Work { public int apply(int x) { return super.apply(x)*3; } }
}
