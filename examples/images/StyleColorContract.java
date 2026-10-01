package org.droidless.images;

import android.app.Activity;
import android.content.res.ColorStateList;
import android.content.res.Resources;
import android.content.res.TypedArray;

/** Theme dispatch, immutable typed-array theme snapshots, colors, GC and faults. */
public final class StyleColorContract {
    static class Context extends Activity {
        Resources.Theme theme;
        int calls;
        boolean fail;
        Context() {
            theme=super.getResources().newTheme(); theme.applyStyle(R.style.PaletteTheme,true);
        }
        public Resources.Theme getTheme() {
            calls++; System.gc();
            if (fail) throw new IllegalStateException("theme callback failed");
            return theme;
        }
    }
    static void check(boolean condition) {
        if (!condition) throw new IllegalStateException("styled color contract");
    }
    public static int run() {
        Context context=new Context();
        int[] attrs={android.R.attr.textColor};
        TypedArray flat=context.obtainStyledAttributes(R.style.InlineText,attrs);
        check(context.calls==1);
        ColorStateList inline=flat.getColorStateList(0);
        check(inline!=null && inline.getDefaultColor()==0xff202a36 && flat.getColor(0,17)==0xff202a36);
        TypedArray old=context.obtainStyledAttributes(R.style.ThemeText,attrs);
        context.theme.applyStyle(R.style.PaletteAlternate,true);
        System.gc();
        ColorStateList states=old.getColorStateList(0);
        check(states.getDefaultColor()==0xff224466 && states.isStateful());
        check(states.getColorForState(new int[]{android.R.attr.state_pressed},0)==0xff123456);
        check(old.getColor(0,17)==0xff224466);
        TypedArray changed=context.obtainStyledAttributes(R.style.ThemeText,attrs);
        check(changed.getColor(0,17)==0xff778899);
        check(old.getColorStateList(0).getDefaultColor()==0xff224466);
        TypedArray cleared=context.obtainStyledAttributes(R.style.NullText,attrs);
        check(cleared.getColorStateList(0)==null && cleared.getColor(0,17)==17);
        TypedArray absent=context.obtainStyledAttributes(R.style.SizeOnlyText,attrs);
        check(absent.getColorStateList(0)==null && absent.getColor(0,19)==19);
        context.fail=true;
        try { context.obtainStyledAttributes(R.style.InlineText,new int[]{android.R.attr.textColor}); throw new IllegalStateException("theme fault ignored"); }
        catch (IllegalStateException expected) { check("theme callback failed".equals(expected.getMessage())); }
        context.fail=false;
        check(context.obtainStyledAttributes(R.style.InlineText,attrs).getColor(0,0)==0xff202a36);
        flat.recycle(); old.recycle(); changed.recycle(); cleared.recycle(); absent.recycle();
        return 1;
    }
    public static TypedArray cycle() {
        Context context=new Context(); context.theme.applyStyle(R.style.PaletteCycle,true);
        return context.obtainStyledAttributes(R.style.ThemeText,new int[]{android.R.attr.textColor});
    }
}
