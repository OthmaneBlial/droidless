package org.droidless.images;

import android.app.Activity;
import android.graphics.Rect;
import android.view.View;
import android.view.ViewGroup;
import android.view.LayoutInflater;
import android.widget.Button;
import android.widget.EditText;
import android.widget.LinearLayout;

/** DEX-driven focus ownership, parent notifications, callback GC and faults. */
public class FocusContract {
    private static Activity host;
    private static Group root, branch;
    private static Probe first, second;
    private static StringBuilder events;
    private static Rect hint;
    private static boolean fail;
    private static int gains, losses;

    private static class Group extends LinearLayout {
        int requests, clears, searches;
        Group(Activity context) { super(context); }
        public void requestChildFocus(View child, View focused) {
            requests++; System.gc(); super.requestChildFocus(child, focused);
        }
        public void clearChildFocus(View child) {
            clears++; System.gc(); super.clearChildFocus(child);
        }
        protected boolean onRequestFocusInDescendants(int direction, Rect rectangle) {
            searches++; System.gc(); return super.onRequestFocusInDescendants(direction, rectangle);
        }
    }
    private static class Probe extends View {
        Probe(Activity context, int id) {
            super(context); setId(id); setFocusable(true);
            setOnFocusChangeListener(new OnFocusChangeListener() {
                public void onFocusChange(View view, boolean gain) {
                    System.gc();
                    if (view != Probe.this || view.isFocused() != gain)
                        throw new AssertionError("listener focus state");
                    if (gain) gains++; else losses++;
                    events.append(getId()).append(gain ? "+;" : "-;");
                }
            });
        }
        protected void onFocusChanged(boolean gain, int direction, Rect rectangle) {
            System.gc();
            if (gain && hint != null && (rectangle != hint || direction != FOCUS_LEFT))
                throw new AssertionError("focus rectangle/direction identity");
            super.onFocusChanged(gain, direction, rectangle);
            if (fail && gain) throw new IllegalStateException("focus callback failure");
        }
    }
    public static int run(Activity context) {
        host = context; events = new StringBuilder(); gains = losses = 0; fail = false;
        root = new Group(context); branch = new Group(context);
        first = new Probe(context, 1); second = new Probe(context, 2);
        root.addView(branch); branch.addView(first); branch.addView(second);
        if (root.isFocusable() || root.hasFocus() || root.findFocus() != null
                || root.getFocusedChild() != null || first.getRootView() != root)
            throw new AssertionError("initial focus metadata");
        if (!new Button(context).isFocusable() || !new EditText(context).isFocusableInTouchMode())
            throw new AssertionError("widget focus defaults");
        LinearLayout inflated = (LinearLayout) LayoutInflater.from(context).inflate(R.layout.focus_probe, null);
        if (inflated.getDescendantFocusability() != ViewGroup.FOCUS_AFTER_DESCENDANTS
                || !inflated.getChildAt(0).isFocusableInTouchMode() || !inflated.requestFocus()
                || inflated.findFocus() != inflated.getChildAt(0))
            throw new AssertionError("XML focusability and descendant policy");
        try {
            branch.setDescendantFocusability(123);
            throw new AssertionError("invalid descendant policy accepted");
        } catch (IllegalArgumentException expected) {
            if (branch.getDescendantFocusability() != ViewGroup.FOCUS_BEFORE_DESCENDANTS)
                throw new AssertionError("invalid policy changed state");
        }
        if (!first.requestFocus() || !first.isFocused() || !branch.hasFocus() || !root.hasFocus()
                || root.isFocused() || root.findFocus() != first || root.getFocusedChild() != branch
                || branch.getFocusedChild() != first || root.requests != 1 || branch.requests != 1)
            throw new AssertionError("initial focus ownership");
        if (!first.requestFocus() || gains != 1 || losses != 0)
            throw new AssertionError("repeated focus duplicated callbacks");
        boolean focusedState = false;
        for (int state : first.getDrawableState()) if (state == android.R.attr.state_focused) focusedState = true;
        if (!focusedState) throw new AssertionError("focused drawable state");
        first.setPressed(true);
        hint = new Rect(1, 2, 3, 4);
        if (!second.requestFocus(View.FOCUS_LEFT, hint) || first.isFocused()
                || root.findFocus() != second || gains != 2 || losses != 1
                || first.isPressed() || !"1+;1-;2+;".equals(events.toString()))
            throw new AssertionError("focus transfer and callback order");
        hint = null;
        second.setFocusable(false);
        if (second.isFocused() || root.findFocus() != first || gains != 3 || losses != 2)
            throw new AssertionError("focusable change/root refocus");
        first.setVisibility(View.GONE);
        if (root.hasFocus() || root.findFocus() != null || losses != 3)
            throw new AssertionError("hidden focus cleanup");
        first.setVisibility(View.VISIBLE); second.setFocusable(true);
        branch.setDescendantFocusability(ViewGroup.FOCUS_BLOCK_DESCENDANTS);
        if (first.requestFocus() || branch.requestFocus() || root.hasFocus())
            throw new AssertionError("blocked descendant focus");
        branch.setFocusable(true);
        if (!branch.requestFocus() || !branch.isFocused() || branch.getFocusedChild() != null
                || root.findFocus() != branch)
            throw new AssertionError("blocked group self focus");
        branch.setDescendantFocusability(ViewGroup.FOCUS_AFTER_DESCENDANTS);
        if (!branch.requestFocus(View.FOCUS_DOWN) || branch.isFocused() || !first.isFocused()
                || branch.searches == 0 || branch.getFocusedChild() != first)
            throw new AssertionError("after-descendants focus");
        branch.setDescendantFocusability(ViewGroup.FOCUS_BEFORE_DESCENDANTS);
        if (!branch.requestFocus() || !branch.isFocused() || first.isFocused())
            throw new AssertionError("before-descendants focus");
        branch.setFocusable(false);
        first.setFocusable(false);
        if (!second.isFocused()) throw new AssertionError("refocus eligible sibling");
        root.removeView(branch);
        if (root.hasFocus() || branch.hasFocus() || second.isFocused() || root.getFocusedChild() != null
                || second.getRootView() != branch || root.clears == 0)
            throw new AssertionError("detached focus cleanup");
        root.addView(branch); first.setFocusable(true); second.setFocusable(true);
        first.setVisibility(View.INVISIBLE);
        if (first.requestFocus() || !root.requestFocus(View.FOCUS_UP) || !second.isFocused())
            throw new AssertionError("reverse descendant search skips hidden children");
        first.setVisibility(View.VISIBLE);
        first.requestFocus(); branch.removeAllViews();
        if (root.hasFocus() || branch.hasFocus() || first.isFocused() || first.getRootView() != first)
            throw new AssertionError("removeAllViews focus and callback GC");
        branch.addView(first); branch.addView(second);
        LinearLayout deepRoot = new LinearLayout(context), cursor = deepRoot;
        for (int i = 0; i < 12; i++) {
            LinearLayout child = new LinearLayout(context); cursor.addView(child); cursor = child;
        }
        Probe leaf = new Probe(context, 3); cursor.addView(leaf);
        if (!deepRoot.requestFocus() || deepRoot.findFocus() != leaf)
            throw new AssertionError("nested focus ownership");
        return 1;
    }
    public static View root() { return root; }
    public static void fault() { fail = true; second.requestFocus(); }
    public static void recover() {
        fail = false;
        if (!second.isFocused() || root.findFocus() != second || !first.requestFocus()
                || second.isFocused() || root.findFocus() != first)
            throw new AssertionError("focus fault recovery");
    }
    public static void cycle() {
        LinearLayout cyclic = new LinearLayout(host); cyclic.addView(cyclic); cyclic.requestFocus();
    }
    public static void startWorkerFocus() {
        new Thread(new Runnable() {
            public void run() { second.requestFocus(); }
        }).start();
    }
    public static boolean workerFocusPreserved() {
        return first.isFocused() && !second.isFocused() && root.findFocus() == first;
    }
    public static void release() { host = null; root = branch = null; first = second = null; events = null; hint = null; }
}
