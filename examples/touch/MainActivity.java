package org.droidless.touch;
import android.app.Activity;
import android.os.Bundle;
import android.view.GestureDetector;
import android.view.MotionEvent;
import android.view.VelocityTracker;
import android.view.View;
import android.widget.Button;
import android.widget.LinearLayout;
import android.widget.FrameLayout;
import android.widget.TextView;

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
    public static int velocityContract() {
        VelocityTracker tracker=VelocityTracker.obtain();
        long start=1L<<54;
        for(int i=0;i<3;i++) {
            MotionEvent event=MotionEvent.obtain(start,start+i*20,i==0?0:2,10+i*20,-i*20,0);
            tracker.addMovement(event);event.recycle();System.gc();
        }
        tracker.computeCurrentVelocity(1000,250);
        if(tracker.getXVelocity()!=250 || tracker.getYVelocity(0)!=-250 || tracker.getXVelocity(99)!=0) throw new AssertionError("velocity clamp/id");
        tracker.computeCurrentVelocity(1);
        if(Math.abs(tracker.getXVelocity()-1)>0.001f || Math.abs(tracker.getYVelocity()+1)>0.001f) throw new AssertionError("velocity units/long clock");
        tracker.clear();tracker.computeCurrentVelocity(1000);
        if(tracker.getXVelocity()!=0 || tracker.getYVelocity()!=0) throw new AssertionError("velocity clear");
        MotionEvent first=MotionEvent.obtain(1,1,0,0,0,0);
        MotionEvent second=MotionEvent.obtain(1,21,2,20,0,0);
        tracker.addMovement(first);tracker.addMovement(second);
        MotionEvent duplicate=MotionEvent.obtain(1,21,2,40,0,0);tracker.addMovement(duplicate);
        tracker.computeCurrentVelocity(1000);
        if(tracker.getXVelocity()!=2000) throw new AssertionError("velocity duplicate timestamp");
        MotionEvent pause=MotionEvent.obtain(1,221,2,40,0,0);tracker.addMovement(pause);tracker.computeCurrentVelocity(1000);
        if(tracker.getXVelocity()!=0) throw new AssertionError("velocity old samples");
        first.recycle();second.recycle();duplicate.recycle();pause.recycle();tracker.recycle();return 1;
    }
    public static void recycledVelocity() { VelocityTracker tracker=VelocityTracker.obtain();tracker.recycle();tracker.clear(); }
    public static void invalidVelocity() { VelocityTracker.obtain().computeCurrentVelocity(1000,Float.NaN); }
    public static int measureContract() {
        FrameLayout row=new FrameLayout(button.getContext());
        LinearLayout column=new LinearLayout(button.getContext());column.setOrientation(1);
        TextView label=new TextView(button.getContext());label.setText("Measured row");
        column.addView(label,new LinearLayout.LayoutParams(-1,-2));
        column.addView(new View(button.getContext()),new LinearLayout.LayoutParams(-1,10));
        row.addView(column,new FrameLayout.LayoutParams(-1,-2));
        if(row.onStartNestedScroll(row,column,2)) throw new AssertionError("base parent accepts nested scrolling");
        row.measure(View.MeasureSpec.makeMeasureSpec(190,View.MeasureSpec.EXACTLY),View.MeasureSpec.makeMeasureSpec(0,View.MeasureSpec.UNSPECIFIED));
        if(row.getMeasuredWidth()!=190 || row.getMeasuredHeight()<54) throw new AssertionError("unbounded row measurement");
        row.measure(View.MeasureSpec.makeMeasureSpec(190,View.MeasureSpec.EXACTLY),View.MeasureSpec.makeMeasureSpec(30,View.MeasureSpec.AT_MOST));
        if(row.getMeasuredHeight()!=30) throw new AssertionError("bounded row measurement");
        row.measure(0,View.MeasureSpec.makeMeasureSpec(0,View.MeasureSpec.EXACTLY));
        if(row.getMeasuredHeight()!=0 || row.getMeasuredWidth()<=0) throw new AssertionError("exact zero and intrinsic match-parent");
        return 1;
    }
}
