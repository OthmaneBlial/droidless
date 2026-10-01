package org.droidless.images;

import android.app.Activity;
import android.graphics.Bitmap;
import android.graphics.BitmapFactory;
import android.graphics.drawable.BitmapDrawable;
import android.os.Bundle;
import android.widget.ImageView;
import android.widget.TextView;
import java.io.InputStream;

/** Exercises packaged PNG, JPEG, and WebP decoding and ImageView rendering. */
public class MainActivity extends Activity {
    @Override public void onCreate(Bundle state) {
        super.onCreate(state);
        BitmapFactory.Options bounds = new BitmapFactory.Options();
        bounds.inJustDecodeBounds = true;
        BitmapFactory.decodeResource(getResources(), R.drawable.sample, bounds);
        if (bounds.outWidth != 96 || bounds.outHeight != 64) {
            throw new IllegalStateException("PNG bounds mismatch");
        }
        bounds.inJustDecodeBounds = false;
        bounds.inSampleSize = 2;
        Bitmap sampled = BitmapFactory.decodeResource(getResources(), R.drawable.sample, bounds);
        if (sampled.getWidth() != 48 || sampled.getHeight() != 32) {
            throw new IllegalStateException("sampled PNG dimensions mismatch");
        }
        bounds.inSampleSize = 1;
        Bitmap png = BitmapFactory.decodeResource(getResources(), R.drawable.sample, bounds);
        Bitmap jpegResource = BitmapFactory.decodeResource(getResources(), R.drawable.sample_jpg);
        Bitmap webpResource = BitmapFactory.decodeResource(getResources(), R.drawable.sample_webp);
        Bitmap jpeg;
        Bitmap webp;
        try {
            InputStream input = getAssets().open("sample.jpg");
            jpeg = BitmapFactory.decodeStream(input);
            input.close();
            input = getAssets().open("sample.webp");
            byte[] encoded = new byte[input.available()];
            input.read(encoded);
            input.close();
            webp = BitmapFactory.decodeByteArray(encoded, 0, encoded.length);
        } catch (Exception error) {
            throw new IllegalStateException(error);
        }
        if (jpegResource.getWidth() != 96 || webpResource.getWidth() != 96
                || jpeg.getHeight() != 64 || webp.getHeight() != 64) {
            throw new IllegalStateException("image dimensions mismatch");
        }
        setContentView(R.layout.main);
        ImageView source = (ImageView) findViewById(R.id.source);
        if (source.getDrawable() == null) throw new IllegalStateException("XML src did not create a Drawable");
        source.setImageResource(R.drawable.sample);
        source.setImageDrawable(getResources().getDrawable(R.drawable.sample));
        source.setImageDrawable(new BitmapDrawable(getResources(), png));
        if (source.getDrawable() == null) throw new IllegalStateException("ImageView lost its Drawable");
        ImageView pngView = (ImageView) findViewById(R.id.png);
        pngView.setImageBitmap(png);
        if (pngView.getDrawable() == null) throw new IllegalStateException("ImageView lost its Bitmap Drawable");
        ((ImageView) findViewById(R.id.jpeg)).setImageBitmap(jpeg);
        ((ImageView) findViewById(R.id.webp)).setImageBitmap(webp);
        ((TextView) findViewById(R.id.status)).setText(
            "PNG " + png.getWidth() + "x" + png.getHeight()
                + " | JPEG " + jpeg.getWidth() + "x" + jpeg.getHeight()
                + " | WebP " + webp.getWidth() + "x" + webp.getHeight()
        );
    }
}
