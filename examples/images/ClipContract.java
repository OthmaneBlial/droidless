package org.droidless.images;
import android.app.Activity;
import android.view.View;
import android.widget.FrameLayout;

/** Painting clips differ from touch bounds; turning off one ancestor cannot undo another clip. */
public final class ClipContract {
    static void check(boolean value) { if (!value) throw new IllegalStateException("clipping contract"); }
    public static View build(Activity activity, int flags) {
        FrameLayout root = new FrameLayout(activity);
        FrameLayout middle = new FrameLayout(activity);
        View leaf = new View(activity);
        check(root.getClipChildren() && root.getClipToPadding());
        check(middle.getClipChildren() && middle.getClipToPadding());
        root.setPadding(10, 10, 10, 10);
        root.setClipChildren((flags & 1) != 0);
        root.setClipToPadding((flags & 2) != 0);
        middle.setClipChildren((flags & 4) != 0);
        middle.setClipToPadding((flags & 8) != 0);
        middle.setPadding(5, 5, 5, 5);
        root.addView(middle, new FrameLayout.LayoutParams(80, 60));
        middle.addView(leaf, new FrameLayout.LayoutParams(100, 80));
        middle.setTranslationX(-20); leaf.setTranslationX(-15);
        activity.setContentView(root);
        leaf.layout(5, 5, 105, 85);
        System.gc();
        check(root.getClipChildren() == ((flags & 1) != 0));
        check(root.getClipToPadding() == ((flags & 2) != 0));
        return root;
    }
    public static View xml(Activity activity) {
        activity.setContentView(R.layout.clipping);
        FrameLayout root = (FrameLayout)activity.findViewById(R.id.clip_root);
        check(!root.getClipChildren() && !root.getClipToPadding());
        return root;
    }
}
