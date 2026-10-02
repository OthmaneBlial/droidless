package org.droidless.images;
import android.app.Activity;
import android.content.Context;
import android.view.View;

/** Virtual metric order, directional boundaries, Java integer wrapping and guest callback faults. */
public final class ScrollQueryContract extends View {
    int offset, range, extent, calls, fail;
    public ScrollQueryContract(Context context) { super(context); }
    static void check(boolean value) { if (!value) throw new IllegalStateException("scroll query contract"); }
    int metric(int step, int value) {
        calls = calls * 10 + step; System.gc();
        if (fail == step) throw new IllegalStateException("scroll metric callback");
        return value;
    }
    protected int computeVerticalScrollOffset() { return metric(1, offset); }
    protected int computeVerticalScrollRange() { return metric(2, range); }
    protected int computeVerticalScrollExtent() { return metric(3, extent); }
    protected int computeHorizontalScrollOffset() { return metric(1, offset); }
    protected int computeHorizontalScrollRange() { return metric(2, range); }
    protected int computeHorizontalScrollExtent() { return metric(3, extent); }
    boolean query(boolean horizontal, int direction) {
        calls = 0;
        boolean result = horizontal ? canScrollHorizontally(direction) : canScrollVertically(direction);
        check(calls == 123); return result;
    }
    public static View run(Activity activity) {
        ScrollQueryContract probe = new ScrollQueryContract(activity);
        probe.layout(0,0,80,60);
        check(probe.superMetrics());
        View plain = new View(activity); plain.layout(0,0,80,60);
        check(!plain.canScrollHorizontally(-1) && !plain.canScrollHorizontally(1));
        check(!plain.canScrollVertically(-1) && !plain.canScrollVertically(1));
        for (boolean horizontal : new boolean[]{false,true}) {
            probe.range=100; probe.extent=40; probe.offset=0;
            check(!probe.query(horizontal,-1) && probe.query(horizontal,1) && probe.query(horizontal,0));
            probe.offset=-5; check(!probe.query(horizontal,-1) && probe.query(horizontal,1));
            probe.offset=1; check(probe.query(horizontal,-1) && probe.query(horizontal,1));
            probe.offset=59; check(probe.query(horizontal,-1) && !probe.query(horizontal,1));
            probe.offset=60; check(!probe.query(horizontal,1));
            probe.range=40; check(!probe.query(horizontal,-1) && !probe.query(horizontal,1));
            probe.range=5; probe.extent=6; probe.offset=1;
            check(probe.query(horizontal,-1) && !probe.query(horizontal,1));
            probe.range=Integer.MAX_VALUE; probe.extent=-1;
            check(probe.query(horizontal,-1) && probe.query(horizontal,1));
            for (int step=1;step<=3;step++) {
                probe.fail=step; boolean caught=false;
                try { probe.query(horizontal,1); } catch (IllegalStateException expected) { caught=true; }
                check(caught && probe.calls==(step==1?1:(step==2?12:123)));
                probe.fail=0; check(probe.query(horizontal,1));
            }
        }
        activity.setContentView(probe); System.gc(); return probe;
    }
    boolean superMetrics() {
        return super.computeHorizontalScrollRange()==80 && super.computeHorizontalScrollExtent()==80
            && super.computeVerticalScrollRange()==60 && super.computeVerticalScrollExtent()==60
            && super.computeHorizontalScrollOffset()==0 && super.computeVerticalScrollOffset()==0;
    }
}
