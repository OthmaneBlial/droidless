package org.droidless.images;

import android.graphics.Paint;
import android.text.TextPaint;

/** API-21 constructor flags, native inheritance and public-field aliases. */
public final class TextPaintContract {
    static class Probe extends TextPaint {
        Probe(int flags) { super(flags); System.gc(); }
    }
    static void check(boolean condition) {
        if (!condition) throw new IllegalStateException("TextPaint state contract");
    }
    public static Paint run() {
        TextPaint plain = new TextPaint();
        check(plain instanceof Paint && plain.getColor()==0xff000000 && plain.getFlags()==0x500);
        check(plain.density==1 && plain.bgColor==0 && plain.baselineShift==0 && plain.linkColor==0 && plain.drawableState==null);
        Paint base = new Paint(3); check(base.getFlags()==0x503);
        Probe paint = new Probe(Paint.ANTI_ALIAS_FLAG | Paint.SUBPIXEL_TEXT_FLAG);
        check(paint.getFlags()==0x581 && paint.density==1);
        paint.density=2.75f; paint.bgColor=7; paint.baselineShift=-3; paint.linkColor=9;
        paint.drawableState=new int[]{1,2};
        TextPaint alias=paint; System.gc();
        check(alias.density==2.75f && alias.bgColor==7 && alias.baselineShift==-3 && alias.linkColor==9 && alias.drawableState==paint.drawableState);
        paint.setColor(0x12345678); paint.setFlags(-1);
        check(paint.getColor()==0x12345678 && paint.getFlags()==-1);
        paint.setFlags(0); System.gc();
        check(paint.getFlags()==0 && paint.getColor()==0x12345678 && paint.drawableState[1]==2);
        return paint;
    }
}
