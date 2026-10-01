package org.droidless.images;

import android.app.Activity;
import android.graphics.Rect;
import android.view.View;
import android.widget.FrameLayout;

/** Authored API-21 integer coordinate contract; not Android-device evidence. */
public final class CoordinateContract {
    static void check(Rect rect, int left, int top, int right, int bottom) {
        if (rect.left != left || rect.top != top || rect.right != right || rect.bottom != bottom)
            throw new IllegalStateException("descendant rectangle coordinates");
    }
    public static int run(Activity host) {
        FrameLayout root = new FrameLayout(host), branch = new FrameLayout(host);
        View child = new View(host);
        root.addView(branch); branch.addView(child);
        root.layout(100, 200, 500, 700); branch.layout(10, 20, 110, 120); child.layout(7, 9, 47, 49);
        branch.setTranslationX(100); // The API-21 integer helper ignores matrices.
        Rect dimensions = new Rect(9, 8, 3, 2);
        if (dimensions.width()!=-6 || dimensions.height()!=-6 || dimensions.centerX()!=6 || dimensions.centerY()!=5)
            throw new IllegalStateException("inverted rectangle dimensions");
        dimensions.set(Integer.MIN_VALUE, Integer.MAX_VALUE, Integer.MAX_VALUE, Integer.MIN_VALUE);
        if (dimensions.width()!=-1 || dimensions.height()!=1) throw new IllegalStateException("wrapped rectangle dimensions");
        dimensions.set(4, 5, 4, 5);
        if (dimensions.width()!=0 || dimensions.height()!=0) throw new IllegalStateException("empty rectangle dimensions");
        Rect rect = new Rect(1, 2, 5, 6);
        System.gc(); root.offsetDescendantRectToMyCoords(child, rect);
        check(rect, 18, 31, 22, 35);
        root.offsetRectIntoDescendantCoords(child, rect); check(rect, 1, 2, 5, 6);
        branch.offsetDescendantRectToMyCoords(child, rect); check(rect, 8, 11, 12, 15);
        branch.offsetRectIntoDescendantCoords(child, rect); check(rect, 1, 2, 5, 6);
        root.offsetDescendantRectToMyCoords(root, null);
        root.offsetRectIntoDescendantCoords(root, rect); check(rect, 1, 2, 5, 6);
        rect.set(Integer.MAX_VALUE, Integer.MIN_VALUE, Integer.MAX_VALUE, Integer.MIN_VALUE);
        root.offsetDescendantRectToMyCoords(child, rect);
        check(rect, Integer.MIN_VALUE + 16, Integer.MIN_VALUE + 29, Integer.MIN_VALUE + 16, Integer.MIN_VALUE + 29);
        root.offsetRectIntoDescendantCoords(child, rect);
        check(rect, Integer.MAX_VALUE, Integer.MIN_VALUE, Integer.MAX_VALUE, Integer.MIN_VALUE);
        try { root.offsetDescendantRectToMyCoords(child, null); throw new IllegalStateException("null rectangle accepted"); }
        catch (NullPointerException expected) {}
        try { root.offsetRectIntoDescendantCoords(null, rect); throw new IllegalStateException("null descendant accepted"); }
        catch (NullPointerException expected) {}
        rect.set(1, 2, 5, 6);
        try { root.offsetDescendantRectToMyCoords(new View(host), rect); throw new IllegalStateException("foreign view accepted"); }
        catch (IllegalArgumentException expected) { check(rect, 1, 2, 5, 6); }
        try { new FrameLayout(host).offsetDescendantRectToMyCoords(child, rect); throw new IllegalStateException("foreign root accepted"); }
        catch (IllegalArgumentException expected) { check(rect, 18, 31, 22, 35); }
        FrameLayout deep = new FrameLayout(host), parent = deep;
        for (int i = 0; i < 12; i++) {
            FrameLayout next = new FrameLayout(host); parent.addView(next); next.layout(3, 4, 23, 24);
            parent = next;
        }
        rect.set(0, 0, 10, 10); System.gc(); deep.offsetDescendantRectToMyCoords(parent, rect);
        check(rect, 36, 48, 46, 58); deep.offsetRectIntoDescendantCoords(parent, rect); check(rect, 0, 0, 10, 10);
        return 1;
    }
}
