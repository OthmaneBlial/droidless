package org.droidless.images;

import android.app.Activity;
import android.view.View;
import android.view.ViewGroup;
import android.widget.FrameLayout;
import android.widget.LinearLayout;
import android.widget.TableLayout;
import android.widget.TableRow;

/** Sized attachment must preserve virtual factories, parameter identity and faults. */
public final class SizedChildContract {
    static class Parent extends LinearLayout {
        ViewGroup.LayoutParams made, seen;
        int factories, additions, index, indexedCalls;
        boolean absent, fail;
        Parent(Activity context) { super(context); }
        protected LinearLayout.LayoutParams generateDefaultLayoutParams() {
            factories++; System.gc();
            if (absent) return null;
            LinearLayout.LayoutParams result=super.generateDefaultLayoutParams();
            made=result; return result;
        }
        public void addView(View child,int at,ViewGroup.LayoutParams params) {
            additions++; index=at; seen=params; System.gc();
            if (fail) throw new IllegalStateException("attachment callback failed");
            super.addView(child,at,params);
        }
        public void addView(View child,int at) {
            indexedCalls++; System.gc(); super.addView(child,at);
        }
        ViewGroup.LayoutParams defaults() { return super.generateDefaultLayoutParams(); }
    }
    static class Frame extends FrameLayout {
        Frame(Activity context) { super(context); }
        ViewGroup.LayoutParams defaults() { return super.generateDefaultLayoutParams(); }
    }
    static class Table extends TableLayout {
        Table(Activity context) { super(context); }
        ViewGroup.LayoutParams defaults() { return super.generateDefaultLayoutParams(); }
    }
    static class Row extends TableRow {
        Row(Activity context) { super(context); }
        ViewGroup.LayoutParams defaults() { return super.generateDefaultLayoutParams(); }
    }
    static void check(boolean value) {
        if (!value) throw new IllegalStateException("sized child contract");
    }
    static class Child extends View {
        int getters;
        Child(Activity context) { super(context); }
        public ViewGroup.LayoutParams getLayoutParams() {
            getters++; System.gc(); return super.getLayoutParams();
        }
    }
    public static View overloads() {
        Activity context=new Activity(); Parent parent=new Parent(context);
        Child first=new Child(context); parent.addView(first);
        check(parent.indexedCalls==1 && parent.additions==1 && parent.factories==1 && first.getters==1);
        check(first.getLayoutParams()==parent.made && parent.seen==parent.made);
        Child second=new Child(context); LinearLayout.LayoutParams kept=new LinearLayout.LayoutParams(23,-2);
        second.setLayoutParams(kept); parent.addView(second,0);
        check(parent.indexedCalls==2 && parent.additions==2 && parent.factories==1 && second.getters==1);
        check(parent.getChildAt(0)==second && parent.seen==kept && second.getLayoutParams()==kept);
        View third=new View(context); LinearLayout.LayoutParams supplied=new LinearLayout.LayoutParams(11,12);
        parent.addView(third,supplied);
        check(parent.indexedCalls==2 && parent.additions==3 && parent.index==-1 && parent.seen==supplied);
        check(parent.getChildCount()==3 && third.getParent()==parent && third.getLayoutParams()==supplied);
        parent.absent=true;
        try { parent.addView(new View(context)); throw new IllegalStateException("null factory accepted"); }
        catch (IllegalArgumentException expected) { check(parent.additions==3 && parent.getChildCount()==3); }
        parent.absent=false; parent.fail=true;
        try { parent.addView(new View(context),supplied); throw new IllegalStateException("attachment fault ignored"); }
        catch (IllegalStateException expected) { check("attachment callback failed".equals(expected.getMessage())); }
        parent.fail=false; parent.addView(new View(context),supplied); System.gc();
        check(parent.getChildCount()==4 && parent.additions==5 && parent.factories==2 && parent.indexedCalls==3);
        return parent;
    }
    public static View run() {
        Activity context=new Activity(); Parent parent=new Parent(context);
        ViewGroup.LayoutParams defaults=parent.defaults();
        check(defaults.width==-2 && defaults.height==-2);
        check(((LinearLayout.LayoutParams)defaults).gravity==-1);
        parent.setOrientation(LinearLayout.VERTICAL);
        defaults=parent.defaults(); check(defaults.width==-1 && defaults.height==-2);
        defaults=new Frame(context).defaults(); check(defaults.width==-1 && defaults.height==-1);
        check(((FrameLayout.LayoutParams)defaults).gravity==-1);
        defaults=new Table(context).defaults(); check(defaults instanceof TableLayout.LayoutParams && defaults.width==-1 && defaults.height==-2);
        defaults=new Row(context).defaults();
        check(defaults instanceof TableRow.LayoutParams && defaults.width==-1 && defaults.height==-2);
        check(((TableRow.LayoutParams)defaults).column==-1 && ((TableRow.LayoutParams)defaults).span==1);
        View child=new View(context); parent.addView(child,73,-2);
        check(parent.factories==1 && parent.additions==1 && parent.index==-1);
        check(parent.made==parent.seen && child.getLayoutParams()==parent.made);
        check(parent.made.width==73 && parent.made.height==-2 && child.getParent()==parent);
        check(parent.getChildCount()==1 && parent.getChildAt(0)==child);
        parent.absent=true;
        try { parent.addView(new View(context),1,2); throw new IllegalStateException("null factory accepted"); }
        catch (NullPointerException expected) { check(parent.additions==1); }
        parent.absent=false; parent.fail=true;
        try { parent.addView(new View(context),-1,0); throw new IllegalStateException("attachment fault ignored"); }
        catch (IllegalStateException expected) { check("attachment callback failed".equals(expected.getMessage())); }
        check(parent.getChildCount()==1 && parent.seen.width==-1 && parent.seen.height==0);
        parent.fail=false; parent.addView(new View(context),-1,0); System.gc();
        check(parent.getChildCount()==2 && parent.factories==4 && parent.additions==3);
        parent.setOrientation(9); check(parent.defaults()==null);
        return parent;
    }
}
