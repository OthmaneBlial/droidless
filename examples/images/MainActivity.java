package org.droidless.images;

import android.app.Activity;
import android.graphics.Bitmap;
import android.graphics.BitmapFactory;
import android.graphics.drawable.BitmapDrawable;
import android.os.Bundle;
import android.content.res.XmlResourceParser;
import android.content.res.TypedArray;
import android.widget.ImageView;
import android.widget.TextView;
import org.xmlpull.v1.XmlPullParser;
import java.io.InputStream;

/** Exercises packaged PNG, JPEG, and WebP decoding and ImageView rendering. */
public class MainActivity extends Activity {
    @Override public void onCreate(Bundle state) {
        super.onCreate(state);
        XmlResourceParser parser = getResources().getXml(R.xml.pull_probe);
        try {
            if (parser.next() != XmlPullParser.START_TAG || !"probe".equals(parser.getName())
                    || parser.getDepth() != 1 || parser.getAttributeCount() != 5
                    || !"parsed".equals(parser.getAttributeValue(0))
                    || parser.getAttributeNameResource(0) != android.R.attr.label
                    || parser.getNamespaceCount(1) != 1
                    || !"http://schemas.android.com/apk/res/android".equals(parser.getNamespaceUri(0))) {
                throw new IllegalStateException("XML pull parser root mismatch");
            }
            int[] requested = {android.R.attr.label, android.R.attr.width,
                android.R.attr.height, android.R.attr.viewportWidth, android.R.attr.alpha};
            TypedArray[] arrays = {
                getResources().obtainAttributes(parser, requested),
                getTheme().obtainStyledAttributes(parser, requested, 0, 0),
                obtainStyledAttributes(parser, requested)
            };
            for (TypedArray array : arrays) {
                if (!"parsed".equals(array.getString(0)) || array.getDimension(1, 0) != 24
                        || array.getDimensionPixelSize(2, 0) != 18
                        || array.getFloat(3, 0) != 24 || array.getFloat(4, 0) != 0.75f) {
                    throw new IllegalStateException("XML typed attributes mismatch");
                }
                array.recycle();
            }
            if (parser.nextTag() != XmlPullParser.START_TAG || !"child".equals(parser.getName())
                    || parser.getDepth() != 2 || !"nested".equals(parser.getAttributeValue(0))
                    || !"android".equals(parser.getAttributePrefix(0))
                    || !"payload".equals(parser.nextText())) {
                throw new IllegalStateException("XML pull parser child mismatch");
            }
            if (parser.nextTag() != XmlPullParser.START_TAG || !"empty".equals(parser.getName())
                    || !parser.isEmptyElementTag()
                    || parser.nextTag() != XmlPullParser.END_TAG
                    || parser.nextTag() != XmlPullParser.END_TAG) {
                throw new IllegalStateException("XML pull parser empty tag mismatch");
            }
            parser.close();
        } catch (Exception error) {
            throw new IllegalStateException("XML pull parser failed", error);
        }
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
        TextView status = (TextView) findViewById(R.id.status);
        status.setTextAppearance(this, R.style.ProbeText);
        status.setText(
            "PNG " + png.getWidth() + "x" + png.getHeight()
                + " | JPEG " + jpeg.getWidth() + "x" + jpeg.getHeight()
                + " | WebP " + webp.getWidth() + "x" + webp.getHeight()
                + " | XML pull OK"
        );
    }
}
