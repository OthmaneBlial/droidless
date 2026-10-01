package org.droidless.documents;

import android.app.Activity;
import android.content.ContentResolver;
import android.content.Intent;
import android.database.Cursor;
import android.graphics.Bitmap;
import android.graphics.BitmapFactory;
import android.net.Uri;
import android.os.Bundle;
import android.provider.DocumentsContract;
import android.view.View;
import android.widget.Button;
import android.widget.ImageView;
import android.widget.LinearLayout;
import android.widget.TextView;
import java.io.InputStream;
import java.util.ArrayList;
import java.util.Collections;
import java.util.ConcurrentModificationException;
import java.util.List;

/** Real host-selected input with guest query, decode, close and GC. */
public class MainActivity extends Activity {
    LinearLayout layout;
    TextView status;
    public void onCreate(Bundle state) {
        super.onCreate(state);
        checkSort();
        checkUri();
        checkImageScale();
        layout = new LinearLayout(this); layout.setOrientation(1);
        status = new TextView(this); status.setText("Choose an image folder");
        status.setTextSize(18); layout.addView(status);
        Button pick = new Button(this); pick.setText("Pick image folder");
        pick.setOnClickListener(new View.OnClickListener() {
            public void onClick(View view) { startActivityForResult(new Intent(Intent.ACTION_OPEN_DOCUMENT_TREE), 404); }
        });
        layout.addView(pick); setContentView(layout);
    }
    static ArrayList<Item> sorting;
    static boolean mutate;
    static class Item implements Comparable<Item> {
        int value, order;
        Item(int value, int order) { this.value = value; this.order = order; }
        public int compareTo(Item other) {
            if (mutate) sorting.clear();
            System.gc(); return value - other.value;
        }
    }
    static void checkSort() {
        sorting = new ArrayList<>();
        sorting.add(new Item(2, 0)); sorting.add(new Item(1, 1)); sorting.add(new Item(1, 2));
        Collections.sort(sorting);
        if (sorting.get(0).order != 1 || sorting.get(1).order != 2 || sorting.get(2).value != 2)
            throw new IllegalStateException("natural sort order/stability");
        Collections.sort(sorting, null);
        mutate = true;
        try { Collections.sort(sorting); throw new IllegalStateException("sort mutation not rejected"); }
        catch (ConcurrentModificationException expected) { }
        mutate = false; sorting = null; System.gc();
    }
    static void checkUri() {
        Uri tree = Uri.parse("content://provider/tree/a%2Fb");
        Uri children = DocumentsContract.buildChildDocumentsUriUsingTree(tree, "c% é");
        if (!children.toString().equals("content://provider/tree/a%2Fb/document/c%25%20%C3%A9/children")
            || !DocumentsContract.getTreeDocumentId(children).equals("a/b")
            || !DocumentsContract.getDocumentId(children).equals("c% é")
            || !children.getAuthority().equals("provider"))
            throw new IllegalStateException("document URI encoding");
        List<String> paths = children.getPathSegments(); System.gc();
        if (paths.size() != 5 || !paths.get(3).equals("c% é"))
            throw new IllegalStateException("URI segments");
        try { paths.add("mutate"); throw new IllegalStateException("mutable URI paths"); }
        catch (UnsupportedOperationException expected) { }
    }
    void checkImageScale() {
        ImageView image = new ImageView(this);
        if (image.getScaleType() != ImageView.ScaleType.FIT_CENTER)
            throw new IllegalStateException("default image scale");
        ImageView.ScaleType[] types = {ImageView.ScaleType.FIT_XY, ImageView.ScaleType.FIT_START,
            ImageView.ScaleType.FIT_CENTER, ImageView.ScaleType.FIT_END, ImageView.ScaleType.CENTER,
            ImageView.ScaleType.CENTER_CROP, ImageView.ScaleType.CENTER_INSIDE};
        for (ImageView.ScaleType type : types) {
            image.setScaleType(type); System.gc();
            if (image.getScaleType() != type) throw new IllegalStateException("image scale identity");
        }
        try { image.setScaleType(null); throw new IllegalStateException("null scale accepted"); }
        catch (NullPointerException expected) { }
    }
    public void onActivityResult(int request, int code, Intent data) {
        if (request != 404) return;
        if (code != RESULT_OK) { status.setText("Folder selection cancelled"); return; }
        Uri tree = data.getData();
        ContentResolver resolver = getContentResolver();
        if (resolver != getApplicationContext().getContentResolver())
            throw new IllegalStateException("resolver identity");
        Uri children = DocumentsContract.buildChildDocumentsUriUsingTree(tree,
            DocumentsContract.getTreeDocumentId(tree));
        Cursor cursor = resolver.query(children, new String[]{"document_id", "mime_type", "_display_name", "_size"}, null, null, null);
        int count = 0;
        try {
            while (cursor.moveToNext()) {
                if (!cursor.getString(1).startsWith("image/")) continue;
                String name = cursor.getString(2);
                Uri document = DocumentsContract.buildDocumentUriUsingTree(tree, cursor.getString(0));
                try {
                    InputStream stream = resolver.openInputStream(document);
                    Bitmap bitmap;
                    try { System.gc(); bitmap = BitmapFactory.decodeStream(stream); }
                    finally { stream.close(); }
                    if (bitmap == null) throw new IllegalStateException("image decode failed");
                    ImageView image = new ImageView(this); image.setImageBitmap(bitmap);
                    image.setContentDescription(name);
                    image.setLayoutParams(new LinearLayout.LayoutParams(-1, 140));
                    layout.addView(image); count++;
                } catch (java.io.IOException error) { throw new IllegalStateException(error); }
            }
        } finally { cursor.close(); }
        System.gc(); status.setText(count + " images · document streams");
    }
}
