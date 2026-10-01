package org.droidless.touch;
import android.app.Activity;
import android.os.Bundle;
import android.view.GestureDetector;
import android.view.MotionEvent;
import android.view.View;
import android.widget.Button;
import android.widget.LinearLayout;

/** Guest callbacks exercise native dispatch, timers, event ownership and collection. */
public class MainActivity extends Activity {
    static int downs, shows, longs, ups, confirms, doubles, doubleEvents, scrolls, flings, clicks, touches;
    static float velocity, distance, localX, rawX;
    static boolean consume, throwing;
    static GestureDetector detector;
    static Button button;
    public void onCreate(Bundle state) {
        super.onCreate(state);
        detector = new GestureDetector(this, new GestureDetector.SimpleOnGestureListener() {
            public boolean onDown(MotionEvent e) { downs++; System.gc(); if(throwing) throw new IllegalStateException("touch callback failed"); return true; }
            public void onShowPress(MotionEvent e) { shows++; System.gc(); }
            public void onLongPress(MotionEvent e) { longs++; System.gc(); }
            public boolean onSingleTapUp(MotionEvent e) { ups++; return true; }
            public boolean onSingleTapConfirmed(MotionEvent e) { confirms++; System.gc(); return true; }
            public boolean onDoubleTap(MotionEvent e) { doubles++; return true; }
            public boolean onDoubleTapEvent(MotionEvent e) { doubleEvents++; return true; }
            public boolean onScroll(MotionEvent a, MotionEvent b, float x, float y) { scrolls++; distance=x; System.gc(); return true; }
            public boolean onFling(MotionEvent a, MotionEvent b, float x, float y) { flings++; velocity=x; System.gc(); return true; }
        });
        LinearLayout root=new LinearLayout(this);root.setOrientation(1);root.setPadding(10,10,10,10);
        View spacer=new View(this);root.addView(spacer,new LinearLayout.LayoutParams(-1,100));
        button=new Button(this);button.setText("Touch button");
        button.setOnClickListener(new View.OnClickListener() {public void onClick(View v) {clicks++;System.gc();}});
        button.setOnTouchListener(new View.OnTouchListener() {
            public boolean onTouch(View v, MotionEvent e) {touches++;localX=e.getX();rawX=e.getRawX();System.gc();return consume;}
        });
        root.addView(button,new LinearLayout.LayoutParams(160,60));setContentView(root);
    }
    public boolean onTouchEvent(MotionEvent e) {detector.onTouchEvent(e);return super.onTouchEvent(e);}
    public static String metrics() {return downs+":"+shows+":"+longs+":"+ups+":"+confirms+":"+doubles+":"+doubleEvents+":"+scrolls+":"+flings+":"+clicks+":"+touches;}
    public static float velocity() {return velocity;}
    public static float distance() {return distance;}
    public static float localX() {return localX;}
    public static float rawX() {return rawX;}
    public static boolean pressed() {return button.isPressed();}
    public static void mode(int mode) {
        consume=mode==1;button.setEnabled(mode!=2);throwing=mode==3;
        button.setTranslationX(mode==4 ? 20 : 0);button.setTranslationY(mode==4 ? 10 : 0);
        button.setAlpha(mode==4 ? 0.5f : 1);
        if(button.getTranslationX()!=(mode==4 ? 20 : 0) || button.getAlpha()!=(mode==4 ? 0.5f : 1)) throw new AssertionError("View properties");
    }
    public static int motionContract() {
        MotionEvent a=MotionEvent.obtain(10,20,MotionEvent.ACTION_MOVE,12.5f,9,3);
        MotionEvent b=MotionEvent.obtain(a);b.offsetLocation(5,-2);
        if(b.getX()!=17.5f || b.getY()!=7 || b.getRawX()!=12.5f || b.getRawY()!=9 || a.getX()!=12.5f
            || b.getEventTime()!=20 || b.getDownTime()!=10 || b.getAction()!=2 || b.getPointerCount()!=1
            || b.getPointerId(0)!=0 || b.findPointerIndex(1)!=-1 || b.getMetaState()!=3) throw new AssertionError("MotionEvent copy/offset");
        b.setLocation(40,50);if(b.getX(0)!=40 || b.getY(0)!=50 || b.getRawX()!=12.5f) throw new AssertionError("MotionEvent setLocation");
        a.recycle();b.recycle();return 1;
    }
}
