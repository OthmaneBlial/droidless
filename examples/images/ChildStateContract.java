package org.droidless.images;

import android.app.Activity;
import android.view.View;
import android.widget.CheckedTextView;
import android.widget.LinearLayout;

/** Child-state capacity, virtual callbacks, GC, notifications and failure recovery. */
public final class ChildStateContract {
    static class Group extends LinearLayout {
        int changes;
        Group(Activity context) { super(context); }
        protected void drawableStateChanged() {
            changes++; System.gc(); super.drawableStateChanged();
        }
        int[] expanded(int extra) { return super.onCreateDrawableState(extra); }
    }
    static class Child extends View {
        int queries, changes;
        boolean fail;
        Child(Activity context) { super(context); }
        protected int[] onCreateDrawableState(int extra) {
            queries++; System.gc();
            if (fail) throw new IllegalStateException("child states failed");
            return new int[]{123, 0};
        }
        protected void drawableStateChanged() { changes++; super.drawableStateChanged(); }
    }
    static void check(boolean condition) {
        if (!condition) throw new IllegalStateException("child state contract");
    }
    public static LinearLayout run(Activity context) {
        Group group = new Group(context);
        Child child = new Child(context);
        CheckedTextView checked = new CheckedTextView(context);
        group.addView(child); group.addView(checked);
        check(!group.addStatesFromChildren());
        check(group.getDrawableState().length==1 && child.queries==0);
        group.setAddStatesFromChildren(true);
        check(group.addStatesFromChildren() && group.changes==1);
        int[] state = group.expanded(2);
        check(child.queries==2 && state.length==7 && state[0]==android.R.attr.state_enabled
            && state[1]==123 && state[2]==android.R.attr.state_enabled && state[3]==0);
        int old=group.changes;
        checked.setChecked(true);
        check(group.changes==old+1);
        boolean found=false;
        for (int value : group.getDrawableState()) if (value==android.R.attr.state_checked) found=true;
        check(found);
        child.fail=true;
        try { group.getDrawableState(); throw new AssertionError("state callback ignored"); }
        catch (IllegalStateException expected) { check("child states failed".equals(expected.getMessage())); }
        child.fail=false; check(group.getDrawableState()[1]==123);
        child.setDuplicateParentStateEnabled(true);
        try { group.refreshDrawableState(); throw new AssertionError("conflicting state modes accepted"); }
        catch (IllegalStateException expected) { }
        group.setAddStatesFromChildren(false);
        check(child.changes==1 && !group.addStatesFromChildren());
        check(child.getDrawableState()[0]==123); // Override remains virtual.
        child.setDuplicateParentStateEnabled(false);
        check(group.getDrawableState().length==1);
        View plain = new View(context); plain.setDuplicateParentStateEnabled(true); group.addView(plain);
        group.setPressed(true);
        int[] duplicated = plain.getDrawableState();
        check(duplicated.length==2 && duplicated[1]==android.R.attr.state_pressed);
        return group;
    }
    public static void cycle(Activity context) {
        LinearLayout group = new LinearLayout(context);
        group.addView(group); group.setAddStatesFromChildren(true); group.getDrawableState();
    }
    public static void queryCycle(Activity context) {
        final LinearLayout group = new LinearLayout(context);
        group.addView(new View(context) {
            protected int[] onCreateDrawableState(int extra) { System.gc(); return group.getDrawableState(); }
        });
        group.setAddStatesFromChildren(true); group.getDrawableState();
    }
    public static void deep(Activity context) {
        LinearLayout group = new LinearLayout(context);
        for (int i=0; i<24; i++) {
            LinearLayout parent = new LinearLayout(context);
            parent.addView(group); parent.setAddStatesFromChildren(true); group=parent;
        }
        group.getDrawableState();
    }

}
