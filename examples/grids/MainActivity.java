package org.droidless.grids;

import android.app.Activity;
import android.os.Bundle;
import android.database.DataSetObserver;
import android.graphics.BitmapFactory;
import android.view.View;
import android.view.ViewGroup;
import android.widget.AdapterView;
import android.widget.BaseAdapter;
import android.widget.GridView;
import android.widget.ImageView;
import android.widget.TextView;

/** Authored adapter contracts, not independent image-app evidence. */
public class MainActivity extends Activity implements AdapterView.OnItemClickListener {
    private static GridView grid;
    private static Photos adapter, oldAdapter;
    private static TextView status;
    private static View failedCell;
    private static View focusedCell;
    private static int focusGains, focusLosses;
    private static int clicks;
    private static long selectedId;
    private static boolean throwClick;
    @Override public void onCreate(Bundle state) {
        super.onCreate(state);
        checkObservers(this);
        setContentView(R.layout.main);
        View plain = findViewById(R.id.plain);
        if (!plain.getClass().getName().equals("android.view.View") || plain.getContext() != this)
            throw new IllegalStateException("plain XML View class/context");
        grid = (GridView) findViewById(R.id.grid);
        status = (TextView) findViewById(R.id.status);
        grid.setOnItemClickListener(this);
        adapter = new Photos(this);
        grid.setAdapter(adapter);
        System.gc();
        if (grid.getAdapter() != adapter || grid.getOnItemClickListener() != this || grid.getCount() != 7
                || grid.getNumColumns() != -1 || grid.getRequestedColumnWidth() != 100
                || grid.getRequestedHorizontalSpacing() != 8 || grid.getVerticalSpacing() != 8)
            throw new IllegalStateException("grid XML values: columns=" + grid.getNumColumns()
                    + ", width=" + grid.getRequestedColumnWidth() + ", spacing=" + grid.getRequestedHorizontalSpacing()
                    + ", vertical=" + grid.getVerticalSpacing() + ", count=" + grid.getCount());
        try { grid.addView(new View(this)); throw new AssertionError("public addView allowed"); }
        catch (UnsupportedOperationException expected) {}
        findViewById(R.id.refresh).setOnClickListener(new View.OnClickListener() {
            public void onClick(View view) { resize(4); status.setText("4 photos · refreshed from adapter"); }
        });
    }
    @Override public void onItemClick(AdapterView<?> parent, View view, int position, long id) {
        System.gc();
        if (parent != grid || view.getParent() != grid || id != 3000000000L + position)
            throw new AssertionError("item callback arguments");
        if (throwClick) throw new IllegalStateException("item callback failed");
        clicks++; selectedId = id;
        status.setText("Selected photo " + (position + 1) + " · id " + id);
    }
    private static class Photos extends BaseAdapter {
        final Activity context;
        int count = 7, mode;
        Photos(Activity context) { this.context = context; }
        public int getCount() { System.gc(); return count; }
        public Object getItem(int position) { return "Photo " + (position + 1); }
        public long getItemId(int position) { System.gc(); return 3000000000L + position; }
        public int getViewTypeCount() { return 2; }
        public int getItemViewType(int position) { return position % 2; }
        public boolean isEnabled(int position) { System.gc(); return position != 1; }
        public View getView(int position, View convertView, ViewGroup parent) {
            System.gc();
            if (mode == 1 && position == 2) throw new IllegalStateException("cell creation failed");
            if (mode == 2 && position == 0) notifyDataSetChanged();
            if (mode == 3 && position == 1) return failedCell;
            ImageView image = new ImageView(context);
            image.setLayoutParams(new GridView.LayoutParams(-1, 96));
            image.setImageBitmap(BitmapFactory.decodeResource(context.getResources(), R.drawable.sample));
            image.setContentDescription("Photo " + (position + 1));
            if (mode != 0 && position == 0) failedCell = image;
            return image;
        }
    }
    private static void checkObservers(Activity context) {
        final Photos watched = new Photos(context);
        final StringBuilder events = new StringBuilder();
        DataSetObserver first = new DataSetObserver() {
            public void onChanged() { System.gc(); events.append("A"); }
            public void onInvalidated() { System.gc(); events.append("!"); }
        };
        DataSetObserver selfRemoving = new DataSetObserver() {
            public void onChanged() { watched.unregisterDataSetObserver(this); System.gc(); events.append("B"); }
        };
        watched.registerDataSetObserver(first); watched.registerDataSetObserver(selfRemoving);
        try { watched.registerDataSetObserver(first); throw new AssertionError("duplicate observer"); }
        catch (IllegalStateException expected) {}
        try { watched.registerDataSetObserver(null); throw new AssertionError("null observer"); }
        catch (IllegalArgumentException expected) {}
        watched.notifyDataSetChanged(); watched.notifyDataSetChanged(); watched.notifyDataSetInvalidated();
        if (!events.toString().equals("BAA!")) throw new AssertionError("reverse observer order or removal");
        try { watched.unregisterDataSetObserver(selfRemoving); throw new AssertionError("unregistered observer"); }
        catch (IllegalStateException expected) {}
        if (watched.isEmpty() || watched.hasStableIds() || !watched.areAllItemsEnabled())
            throw new AssertionError("BaseAdapter defaults");
    }
    public static int clickCount() { return clicks; }
    public static long selectedId() { return selectedId; }
    public static void resize(int count) { adapter.count = count; adapter.notifyDataSetChanged(); }
    public static void keepFocusCell(View cell) {
        focusedCell = cell; focusGains = focusLosses = 0;
        cell.setOnFocusChangeListener(new View.OnFocusChangeListener() {
            public void onFocusChange(View view, boolean gain) {
                System.gc();
                if (view != focusedCell || view.isFocused() != gain || view.getParent() != grid)
                    throw new AssertionError("grid focus callback identity/state/parent");
                if (gain) focusGains++; else focusLosses++;
            }
        });
    }
    public static int focusChanges() { return 10 * focusGains + focusLosses; }
    public static void releaseFocusCell() { focusedCell = null; }
    public static void invalidate() { adapter.notifyDataSetInvalidated(); }
    public static void failBind(int mode) { adapter.mode = mode; adapter.notifyDataSetChanged(); }
    public static View takeFailedCell() { View cell = failedCell; failedCell = null; return cell; }
    public static void recover() { adapter.mode = 0; adapter.count = 7; adapter.notifyDataSetChanged(); }
    public static void replace() { oldAdapter = adapter; adapter = new Photos(oldAdapter.context); adapter.count = 3; grid.setAdapter(adapter); }
    public static void notifyOld() { oldAdapter.count = 9; oldAdapter.notifyDataSetChanged(); }
    public static void detach() { grid.setAdapter(null); }
    public static void reattach() { grid.setAdapter(adapter); }
    public static void throwClick(boolean enabled) { throwClick = enabled; }
    public static void geometry(int mode) { grid.setNumColumns(2); grid.setColumnWidth(80); grid.setHorizontalSpacing(12); grid.setStretchMode(mode); }
    public static String metrics() { return grid.getNumColumns() + ":" + grid.getColumnWidth() + ":" + grid.getHorizontalSpacing(); }
}
