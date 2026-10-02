package org.droidless.images;

import android.app.Activity;
import android.graphics.Paint;
import android.graphics.Typeface;
import android.text.TextPaint;
import android.view.View;
import android.widget.TextView;

/** Cached family/style identities, null defaults, Paint state and native View metadata. */
public final class TypefaceContract {
    static void check(boolean value) {
        if (!value) throw new IllegalStateException("typeface contract");
    }
    public static View run() {
        Typeface mono=Typeface.create("monospace",Typeface.BOLD_ITALIC);
        check(mono.getStyle()==3 && mono.isBold() && mono.isItalic());
        check(Typeface.create(mono,3)==mono && Typeface.create("monospace",3)==mono);
        check(Typeface.create(mono,99)==Typeface.MONOSPACE);
        check(Typeface.create((Typeface)null,1)==Typeface.DEFAULT_BOLD);
        check(Typeface.create("unknown-family",0)==Typeface.DEFAULT);
        check(Typeface.SANS_SERIF==Typeface.DEFAULT && Typeface.SERIF!=Typeface.DEFAULT);
        for(int style=0;style<4;style++) check(Typeface.defaultFromStyle(style).getStyle()==style);
        try { Typeface.defaultFromStyle(-1); throw new IllegalStateException("bad style accepted"); }
        catch(ArrayIndexOutOfBoundsException expected) {}
        Paint paint=new TextPaint(); check(paint.getTypeface()==null);
        check(paint.setTypeface(mono)==mono); System.gc(); check(paint.getTypeface()==mono);
        check(paint.setTypeface(Typeface.SERIF)==Typeface.SERIF && paint.getTypeface()==Typeface.SERIF);
        check(paint.setTypeface(null)==null && paint.getTypeface()==null);
        TextView text=new TextView(new Activity()); text.setText("Native mono bold italic");
        text.setTypeface(mono); System.gc(); check(text.getTypeface()==mono);
        text.measure(View.MeasureSpec.makeMeasureSpec(200,View.MeasureSpec.EXACTLY),View.MeasureSpec.makeMeasureSpec(100,View.MeasureSpec.EXACTLY));
        android.text.Layout layout=text.getLayout(); text.setTypeface(mono); check(text.getLayout()==layout);
        text.setTypeface(null); check(text.getTypeface()==null && text.getLayout()==null);
        text.setTypeface(mono); return text;
    }
}
