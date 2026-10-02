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
        int factories, additions, index;
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
