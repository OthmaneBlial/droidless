package org.droidless.images;

import android.graphics.Paint;
import android.graphics.Typeface;
import android.text.TextPaint;

/** Real host-font metrics, retained size/face and collection safety. */
public final class FontMetricsContract {
    static void check(boolean value) {
        if (!value) throw new IllegalStateException("font metrics contract");
    }
    public static void run() {
        Paint paint=new TextPaint(); check(paint.getTextSize()==12);
        Typeface[] families={Typeface.DEFAULT,Typeface.SERIF,Typeface.MONOSPACE};
        for(Typeface family:families) for(int style=0;style<4;style++) {
            paint.setTypeface(Typeface.create(family,style)); paint.setTextSize(20);
            float ascent=paint.ascent(),descent=paint.descent();
            check(ascent<0 && descent>0); System.gc();
            check(paint.ascent()==ascent && paint.descent()==descent);
            paint.setTextSize(40);
            check(Math.abs(paint.ascent()-2*ascent)<0.01f && Math.abs(paint.descent()-2*descent)<0.01f);
            paint.setTextSize(-1); check(paint.getTextSize()==40);
            paint.setTextSize(0); check(paint.ascent()==0 && paint.descent()==0);
        }
        paint.setTypeface(null); paint.setTextSize(20); check(paint.ascent()<0 && paint.descent()>0);
    }
}
