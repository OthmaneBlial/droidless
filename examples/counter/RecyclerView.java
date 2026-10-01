package android.support.v7.widget;

import android.app.Activity;
import android.widget.FrameLayout;

/** Authored guard probe, not the support library or independent APK evidence. */
public class RecyclerView extends FrameLayout {
    public static class ItemAnimator {}
    private ItemAnimator animator = new ItemAnimator();
    public int clears;
    public RecyclerView(Activity context) { super(context); }
    public void setItemAnimator(ItemAnimator value) {
        animator = value;
        clears++;
        System.gc();
    }
    @Override protected void onMeasure(int width, int height) {
        if (animator != null) throw new IllegalStateException("animator policy applied too late");
        System.gc();
        super.onMeasure(width, height);
    }
}
