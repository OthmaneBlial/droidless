package org.droidless.touch;

import android.app.Activity;
import android.content.Context;
import android.content.res.ColorStateList;
import android.graphics.Rect;
import android.graphics.RectF;
import android.graphics.Matrix;
import android.graphics.drawable.ColorDrawable;
import android.graphics.drawable.Drawable;
import android.graphics.PorterDuff;
import android.util.AttributeSet;
import android.view.LayoutInflater;
import android.view.Gravity;
import android.view.View;
import android.view.ViewGroup;
import android.view.ViewStub;
import android.widget.FrameLayout;
import android.widget.TableLayout;
import android.widget.TableRow;
import android.widget.CheckedTextView;

/** Guest metadata and geometry must drive both the rendered tree and root touch. */
public class CustomLayout extends FrameLayout {
    static int generated, measured, laidOut, primary, cover, mode;
    static boolean fail;
    static int drawableChanges, childChanges;
    static CustomLayout current;
    public CustomLayout(Context context, AttributeSet attrs) { super(context, attrs); }
    protected int[] onCreateDrawableState(int extra) {
        System.gc();return mergeDrawableStates(super.onCreateDrawableState(extra+1),new int[]{android.R.attr.state_selected});
    }
    protected void drawableStateChanged() { drawableChanges++;System.gc();super.drawableStateChanged(); }
    public void childDrawableStateChanged(View child) { childChanges++;System.gc();super.childDrawableStateChanged(child); }
    static boolean hasState(View view,int state) { for(int value:view.getDrawableState()) if(value==state)return true;return false; }
    static class InsetsProbe extends View {
        InsetsProbe(Activity activity) { super(activity); }
        boolean apply(Rect insets) { return fitSystemWindows(insets); }
    }
    static class ReverseList extends java.util.ArrayList<Object> {
        int writes;
        public Object set(int index,Object value) { writes++;System.gc();return super.set(index,value); }
    }
    static class Params extends FrameLayout.LayoutParams {
        int marker = 7;
        Params(Context context, AttributeSet attrs) { super(context, attrs); }
    }
    public FrameLayout.LayoutParams generateLayoutParams(AttributeSet attrs) {
        if (getChildCount()!=generated) throw new AssertionError("incremental child attachment");
        generated++;
        System.gc();
        return new Params(getContext(), attrs);
    }
    protected void onMeasure(int width, int height) {
        measured++;
        super.onMeasure(width, height);
        for (int i=0; i<getChildCount(); i++) {
            View child=getChildAt(i);
            Params params=(Params)child.getLayoutParams();
            if (params.marker!=7 || params.width!=100 || params.height!=50) throw new AssertionError("guest layout metadata");
            child.measure(MeasureSpec.makeMeasureSpec(params.width,MeasureSpec.EXACTLY),MeasureSpec.makeMeasureSpec(params.height,MeasureSpec.EXACTLY));
        }
    }
    protected void onLayout(boolean changed, int left, int top, int right, int bottom) {
        laidOut++;
        System.gc();
        if (fail) throw new IllegalStateException("custom layout failed");
        getChildAt(0).layout(10,20,110,70);
        int x=mode==0 ? -100 : 10;
        getChildAt(1).layout(x,20,x+100,70);
    }
    static void install(Activity activity) {
        generated=measured=laidOut=primary=cover=mode=0;fail=false;
        FrameLayout parent=new FrameLayout(activity);
        current=(CustomLayout)LayoutInflater.from(activity).inflate(R.layout.overlay,parent,false);
        if (parent.getChildCount()!=0 || !(current.getLayoutParams() instanceof FrameLayout.LayoutParams)
            || current.getLayoutParams().width!=-1 || current.isChildrenDrawingOrderEnabled()) throw new AssertionError("unattached root metadata/default order");
        current.getChildAt(0).setOnClickListener(new View.OnClickListener() {
            public void onClick(View view) { primary++;System.gc(); }
        });
        current.getChildAt(1).setOnClickListener(new View.OnClickListener() {
            public void onClick(View view) { cover++;System.gc(); }
        });
        activity.setContentView(current);
    }
    static void configure(int value) {
        mode=value;fail=value==2;
        current.setTranslationX(0.5f);
        current.requestLayout();
    }
    static String state() { return generated+":"+measured+":"+laidOut+":"+primary+":"+cover; }
    static int metadataContract(Activity activity) {
        ReverseList reverse=new ReverseList();Object first=new Object(),last=new Object();
        reverse.add(first);reverse.add(null);reverse.add(last);
        java.util.Iterator<Object> retained=reverse.iterator();java.util.Collections.reverse(reverse);System.gc();
        if (reverse.get(0)!=last || reverse.get(1)!=null || reverse.get(2)!=first || reverse.writes!=2
            || retained.next()!=last || retained.next()!=null || retained.next()!=first) throw new AssertionError("virtual reverse/GC/nonstructural iteration");
        int rejected=0;
        try { java.util.Collections.reverse(java.util.Collections.unmodifiableList(reverse)); } catch (UnsupportedOperationException expected) { rejected++; }
        java.util.Collections.reverse(new java.util.ArrayList<Object>());
        if (rejected!=1 || reverse.get(0)!=last) throw new AssertionError("reverse read-only/empty");
        float nan=0.0f/0.0f;
        if (Math.min(1.0f,2.0f)!=1.0f || Math.min(2.0f,1.0f)!=1.0f
            || 1.0f/Math.min(0.0f,-0.0f)!=Float.NEGATIVE_INFINITY || 1.0f/Math.min(-0.0f,0.0f)!=Float.NEGATIVE_INFINITY
            || Math.min(nan,1.0f)==Math.min(nan,1.0f) || Math.min(1.0f,nan)==Math.min(1.0f,nan)) throw new AssertionError("float minimum/negative zero/NaN");
        android.util.SparseIntArray indices=new android.util.SparseIntArray();
        if (indices.indexOfKey(0)!=-1) throw new AssertionError("empty sparse insertion point");
        indices.put(42,1);indices.put(Integer.MIN_VALUE,2);indices.put(Integer.MAX_VALUE,3);indices.put(42,4);System.gc();
        if (indices.indexOfKey(Integer.MIN_VALUE)!=0 || indices.indexOfKey(42)!=1 || indices.indexOfKey(Integer.MAX_VALUE)!=2
            || indices.indexOfKey(0)!=-2 || indices.indexOfKey(43)!=-3 || indices.size()!=3 || indices.get(42)!=4) throw new AssertionError("sparse signed rank/insertion/replacement");
        indices.delete(42);
        if (indices.indexOfKey(42)!=-2 || indices.indexOfKey(Integer.MAX_VALUE)!=1) throw new AssertionError("sparse deletion rank");
        android.util.SparseArray<Object> objects=new android.util.SparseArray<Object>();objects.put(9,activity);
        if (objects.indexOfKey(9)!=0 || objects.indexOfKey(10)!=-2) throw new AssertionError("shared object sparse rank");
        if (activity.getClassLoader()!=CustomLayout.class.getClassLoader()) throw new AssertionError("Context APK class loader");
        if (Gravity.getAbsoluteGravity(Gravity.START|Gravity.TOP,0)!=(Gravity.LEFT|Gravity.TOP)
            || Gravity.getAbsoluteGravity(Gravity.START|Gravity.BOTTOM,1)!=(Gravity.RIGHT|Gravity.BOTTOM)
            || Gravity.getAbsoluteGravity(Gravity.END|Gravity.TOP,1)!=(Gravity.LEFT|Gravity.TOP)
            || Gravity.getAbsoluteGravity(Gravity.CENTER,0)!=Gravity.CENTER) throw new AssertionError("relative gravity");
        Rect container=new Rect(10,20,110,220),out=new Rect();
        Gravity.apply(Gravity.START|Gravity.BOTTOM,30,40,container,out,1);
        if (out.left!=80 || out.right!=110 || out.top!=180 || out.bottom!=220) throw new AssertionError("gravity RTL placement");
        Gravity.apply(Gravity.CENTER,30,40,container,out);
        if (out.left!=45 || out.right!=75 || out.top!=100 || out.bottom!=140) throw new AssertionError("gravity center placement");
        Gravity.apply(Gravity.CENTER|Gravity.CLIP_HORIZONTAL|Gravity.CLIP_VERTICAL,300,400,container,out);
        if (out.left!=10 || out.right!=110 || out.top!=20 || out.bottom!=220) throw new AssertionError("gravity clip");
        Gravity.apply(Gravity.FILL,30,40,container,container);
        if (container.left!=10 || container.right!=110 || container.top!=20 || container.bottom!=220) throw new AssertionError("gravity fill alias");
        container.left=30;Gravity.apply(Gravity.LEFT|Gravity.TOP,10,10,container,out);
        if (out.left!=30 || out.top!=20) throw new AssertionError("Rect guest fields");
        out.setEmpty();
        if (out.left!=0 || out.right!=0 || out.top!=0 || out.bottom!=0 || out.contains(0,0)) throw new AssertionError("Rect empty");
        out.set(0,0,100,100);
        if (out.intersects(100,0,120,20) || out.intersect(100,0,120,20) || out.right!=100
            || !out.intersects(80,90,120,140) || out.left!=0 || !out.intersect(80,90,120,140)
            || out.left!=80 || out.top!=90 || out.right!=100 || out.bottom!=100) throw new AssertionError("Rect intersection");
        out.set(-10,-20,-1,-3);
        if (out.centerX()!=-6 || out.centerY()!=-12) throw new AssertionError("Rect center rounding");
        if (!out.contains(-10,-20) || !out.contains(-2,-4) || out.contains(-1,-4)
            || out.contains(-2,-3) || out.contains(-11,-20)) throw new AssertionError("Rect exclusive edges");
        out.left=0;
        if (out.contains(-2,-4)) throw new AssertionError("Rect inverted guest fields");
        FrameLayout parent=new FrameLayout(activity);
        if (parent.getWindowSystemUiVisibility()!=0) throw new AssertionError("unconfigured system UI visibility");
        InsetsProbe insetsProbe=new InsetsProbe(activity);Rect insets=new Rect(1,2,3,4);
        if (insetsProbe.apply(insets) || insets.left!=1 || insetsProbe.apply(null)) throw new AssertionError("unconsumed insets");
        insetsProbe.setFitsSystemWindows(true);
        if (!insetsProbe.apply(insets) || insets.left!=0 || insets.bottom!=0
            || insetsProbe.getPaddingLeft()!=1 || insetsProbe.getPaddingTop()!=2
            || insetsProbe.getPaddingRight()!=3 || insetsProbe.getPaddingBottom()!=4) throw new AssertionError("legacy padding insets");
        if (parent.getElevation()!=0) throw new AssertionError("default elevation");
        CheckedTextView checked=new CheckedTextView(activity);
        checked.setText("Checked label");checked.setDuplicateParentStateEnabled(true);
        if (checked.isChecked()) throw new AssertionError("default checked state");
        checked.setChecked(true);System.gc();
        if (!checked.isChecked() || !"Checked label".equals(checked.getText())) throw new AssertionError("checked text state");
        checked.setChecked(false);
        if (checked.isChecked()) throw new AssertionError("cleared checked state");
        if (parent.getBaseline()!=-1 || checked.getBaseline()!=-1) throw new AssertionError("unmeasured baseline");
        checked.measure(MeasureSpec.makeMeasureSpec(100,MeasureSpec.EXACTLY),MeasureSpec.makeMeasureSpec(50,MeasureSpec.EXACTLY));
        if (checked.getBaseline()<0 || checked.getBaseline()>=checked.getMeasuredHeight()) throw new AssertionError("bounded text baseline");
        if (parent.getFocusedChild()!=null) throw new AssertionError("unfocused group child");
        if (parent.isFocusable() || parent.isFocusableInTouchMode()) throw new AssertionError("default focus");
        parent.setFocusableInTouchMode(true);
        if (!parent.isFocusable() || !parent.isFocusableInTouchMode()) throw new AssertionError("touch focus enables focus");
        parent.setFocusable(false);
        if (parent.getFocusedChild()!=null) throw new AssertionError("unfocused group child");
        if (parent.isFocusable() || parent.isFocusableInTouchMode()) throw new AssertionError("focus disabled");
        parent.setElevation(3.5f);
        if (parent.getElevation()!=3.5f || parent.getZ()!=3.5f) throw new AssertionError("stored elevation");
        Matrix translation=parent.getMatrix();
        if (!translation.isIdentity() || translation!=parent.getMatrix()) throw new AssertionError("View matrix identity");
        parent.setTranslationX(1.5f);parent.setTranslationY(-2.5f);
        RectF mapped=new RectF(10,20,40,60);
        if (translation!=parent.getMatrix() || !translation.mapRect(mapped) || mapped.left!=11.5f || mapped.top!=17.5f
            || mapped.right!=41.5f || mapped.bottom!=57.5f) throw new AssertionError("View matrix translation");
        translation.setScale(-2,3);mapped.set(new Rect(10,20,40,60));mapped.left=12;
        if (!translation.mapRect(mapped) || mapped.left!=-80 || mapped.right!=-24 || mapped.top!=60 || mapped.bottom!=180) throw new AssertionError("RectF scale bounds");
        parent.setTranslationX(0);parent.setTranslationY(0);
        int[] modes={MeasureSpec.EXACTLY,MeasureSpec.AT_MOST,MeasureSpec.UNSPECIFIED};
        int[] dims={30,-1,-2};
        for (int m:modes) for (int d:dims) {
            int spec=ViewGroup.getChildMeasureSpec(MeasureSpec.makeMeasureSpec(100,m),20,d);
            int expectedMode=d>=0 ? MeasureSpec.EXACTLY : m==MeasureSpec.EXACTLY && d==-1 ? MeasureSpec.EXACTLY : m==MeasureSpec.UNSPECIFIED ? m : MeasureSpec.AT_MOST;
            int expectedSize=d>=0 ? d : m==MeasureSpec.UNSPECIFIED ? 0 : 80;
            if (MeasureSpec.getMode(spec)!=expectedMode || MeasureSpec.getSize(spec)!=expectedSize) throw new AssertionError("child measure spec");
        }
        CustomLayout probe=new CustomLayout(activity,null);
        drawableChanges=childChanges=0;
        probe.addView(checked);checked.setChecked(true);probe.refreshDrawableState();
        checked.setTextColor(ColorStateList.valueOf(0xff123456));
        Drawable start=new ColorDrawable(0xff123456),end=new ColorDrawable(0xff123456);
        start.setBounds(0,0,18,18);start.setTintList(ColorStateList.valueOf(0xff112233));start.setTintMode(PorterDuff.Mode.SRC_IN);
        checked.setCompoundDrawablesRelative(start,null,end,null);start=end=null;System.gc();
        Drawable[] slots=checked.getCompoundDrawablesRelative();
        if (slots.length!=4 || slots[0]==null || slots[2]==null || slots[0]==slots[2] || slots[1]!=null || slots[3]!=null
            || slots[0].getBounds().right!=18 || checked.getCompoundDrawablesRelative()[0]!=slots[0]) throw new AssertionError("compound drawable identity/bounds/GC");
        if (checked.getCompoundDrawables()[0]!=slots[0]) throw new AssertionError("LTR absolute compound slot");
        checked.setCompoundDrawables(slots[0],slots[2],null,null);System.gc();
        if (checked.getCompoundDrawablesRelative()[0]!=null || checked.getCompoundDrawablesRelative()[2]!=null
            || checked.getCompoundDrawablesRelative()[1]!=slots[2] || checked.getCompoundDrawables()[0]!=slots[0]) throw new AssertionError("absolute setter clears relative edges/preserves top");
        slots[0].setTintList(null);checked.setCompoundDrawables(null,null,null,null);System.gc();
        if (checked.getCompoundDrawablesRelative()[0]!=null) throw new AssertionError("compound drawable clear");
        if (drawableChanges!=1 || childChanges!=1 || !hasState(checked,android.R.attr.state_checked)
            || !hasState(checked,android.R.attr.state_selected) || checked.getCurrentTextColor()!=0xff123456) throw new AssertionError("guest drawable callbacks/parent state/color");
        checked.setChecked(false);
        if (childChanges!=2 || hasState(checked,android.R.attr.state_checked)) throw new AssertionError("checked state refresh");
        probe.setMinimumWidth(12);probe.setMinimumHeight(14);
        probe.setBackgroundColor(0xff112233);
        if (probe.getSuggestedMinimumWidth()!=12 || probe.getSuggestedMinimumHeight()!=14) throw new AssertionError("suggested minimum");
        probe.setMeasuredDimension(0x01000032,0x02000046);
        if (probe.getMeasuredWidth()!=50 || probe.getMeasuredHeight()!=70 || probe.getMeasuredWidthAndState()!=0x01000032
            || probe.getMeasuredHeightAndState()!=0x02000046 || probe.getMeasuredState()!=0x01000200) throw new AssertionError("measured state");
        if (View.combineMeasuredStates(0x01000000,0x00000200)!=0x01000200
            || View.resolveSizeAndState(120,MeasureSpec.makeMeasureSpec(100,MeasureSpec.AT_MOST),0x02000000)!=0x03000064
            || View.resolveSizeAndState(120,MeasureSpec.makeMeasureSpec(100,MeasureSpec.EXACTLY),0)!=100
            || View.resolveSizeAndState(120,0,0)!=120) throw new AssertionError("resolved state");
        View child=new View(activity);
        FrameLayout.LayoutParams margins=new FrameLayout.LayoutParams(-1,-1);
        margins.leftMargin=3;margins.rightMargin=5;margins.topMargin=7;margins.bottomMargin=9;
        if (margins.getMarginStart()!=3 || margins.getMarginEnd()!=5) throw new AssertionError("absolute margin fallback");
        child.setLayoutParams(margins);probe.setPadding(1,2,3,4);
        if (probe.getPaddingLeft()!=1 || probe.getPaddingTop()!=2 || probe.getPaddingRight()!=3 || probe.getPaddingBottom()!=4) throw new AssertionError("per-edge padding");
        probe.measureChildWithMargins(child,MeasureSpec.makeMeasureSpec(100,MeasureSpec.EXACTLY),10,
            MeasureSpec.makeMeasureSpec(200,MeasureSpec.EXACTLY),20);
        if (child.getMeasuredWidth()!=78 || child.getMeasuredHeight()!=158) throw new AssertionError("margin measurement");
        View result=LayoutInflater.from(activity).inflate(R.layout.overlay_piece,parent,true);
        if (result!=parent || parent.getChildCount()!=1 || parent.getChildAt(0).getLayoutParams().width!=100) throw new AssertionError("merge attachment");
        TableLayout table=(TableLayout)LayoutInflater.from(activity).inflate(R.layout.table,null);
        TableRow row=(TableRow)table.getChildAt(0);
        ColorStateList palette=activity.getResources().getColorStateList(R.color.text_states);
        if (!palette.isStateful() || palette.getDefaultColor()!=0xff123456
            || palette.getColorForState(new int[]{android.R.attr.state_enabled},0)!=0xff123456
            || palette.getColorForState(new int[]{},0)!=0xff654321
            || ((android.widget.TextView)row.getChildAt(0)).getCurrentTextColor()!=0xff123456) throw new AssertionError("theme color selector/default/negative state");
        if (!(table.getChildAt(1) instanceof ViewStub) || table.getChildAt(1).getVisibility()!=View.GONE) throw new AssertionError("ViewStub XML class/default visibility");
        ViewStub stub=(ViewStub)table.getChildAt(1);
        ViewGroup.LayoutParams stubParams=stub.getLayoutParams();View expanded=stub.inflate();System.gc();
        if (table.getChildCount()!=2 || table.getChildAt(1)!=expanded || expanded.getParent()!=table || stub.getParent()!=null
            || expanded.getLayoutParams()!=stubParams || expanded.getId()!=R.id.stub_expanded) throw new AssertionError("ViewStub replacement/index/params/id");
        stub.setVisibility(View.GONE);
        if (expanded.getVisibility()!=View.GONE) throw new AssertionError("inflated visibility");
        stub.setVisibility(View.VISIBLE);
        if (expanded.getVisibility()!=View.VISIBLE) throw new AssertionError("inflated visibility restored");
        try { stub.inflate();throw new AssertionError("ViewStub inflated twice"); } catch (IllegalStateException expected) {}
        if (table.hasTransientState() || expanded.hasTransientState()) throw new AssertionError("default transient state");
        expanded.setHasTransientState(true);expanded.setHasTransientState(true);System.gc();
        expanded.setHasTransientState(false);
        if (!expanded.hasTransientState() || !table.hasTransientState()) throw new AssertionError("counted/child transient state");
        expanded.setHasTransientState(false);expanded.setHasTransientState(false);
        if (expanded.hasTransientState() || table.hasTransientState()) throw new AssertionError("cleared transient state/underflow");
        expanded.setHasTransientState(true);
        table.removeViewAt(1);System.gc();
        if (table.hasTransientState() || !expanded.hasTransientState()) throw new AssertionError("detached transient child");
        if (table.getChildCount()!=1 || expanded.getParent()!=null) throw new AssertionError("indexed removal parent link");
        try { table.removeViewAt(-1);throw new AssertionError("negative child index accepted"); } catch (IndexOutOfBoundsException expected) {}
        try { table.removeViewAt(1);throw new AssertionError("out-of-bounds child index accepted"); } catch (IndexOutOfBoundsException expected) {}
        if (table.getChildCount()!=1) throw new AssertionError("indexed removal fault mutation");
        if (!(row.getLayoutParams() instanceof TableLayout.LayoutParams) || row.getLayoutParams().width!=-1 || row.getLayoutParams().height!=-2
            || !(row.getChildAt(0).getLayoutParams() instanceof TableRow.LayoutParams) || row.getChildAt(0).getLayoutParams().width!=-1
            || row.getChildAt(0).getLayoutParams().height!=-2) throw new AssertionError("table metadata defaults");
        System.gc();return 1;
    }
    static void mergeWithoutParent(Activity activity) { LayoutInflater.from(activity).inflate(R.layout.overlay_piece,null); }
}
