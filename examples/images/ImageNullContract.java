package org.droidless.images;
import android.app.Activity;
import android.widget.ImageView;
import android.view.View;

/** Explicit XML null sources and programmatic replacement/clearing preserve Drawable identity. */
public final class ImageNullContract {
    static void check(boolean value) { if (!value) throw new IllegalStateException("null image contract"); }
    public static View run(Activity activity) {
        activity.setContentView(R.layout.null_images);
        ImageView[] views = {(ImageView)activity.findViewById(R.id.null_src), (ImageView)activity.findViewById(R.id.null_compat)};
        for (ImageView view : views) {
            check(view.getDrawable() == null);
            view.setImageResource(R.drawable.sample); check(view.getDrawable() != null);
            System.gc(); view.setImageResource(0); check(view.getDrawable() == null);
        }
        return (View)activity.findViewById(R.id.null_root);
    }
}
