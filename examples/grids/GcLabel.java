package org.droidless.grids;
import android.content.Context;
import android.util.AttributeSet;
import android.widget.TextView;
/** Force collection while the inflater still owns the unfinished parent tree. */
public class GcLabel extends TextView {
    static { System.gc(); }
    public GcLabel(Context context, AttributeSet attributes) { super(context, attributes); System.gc(); }
}
