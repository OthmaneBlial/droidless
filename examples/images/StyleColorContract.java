package org.droidless.images;

import android.app.Activity;
import android.content.res.ColorStateList;
import android.content.res.Resources;
import android.content.res.TypedArray;
import android.widget.TextView;
import android.view.View;

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
    static class Probe extends TextView {
        int colors;
        ColorStateList seen;
        boolean fail;
        Probe(Context context) { super(context); }
        public void setTextColor(ColorStateList value) {
            colors++; System.gc();
            if (fail) throw new IllegalStateException("appearance callback failed");
            seen=value; super.setTextColor(value);
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
    public static TextView appearance() {
        Context owner=new Context(), other=new Context();
        other.theme.applyStyle(R.style.PaletteAlternate,true);
        Probe view=new Probe(owner);
        view.setText("Measured label");
        view.setTextAppearance(other,R.style.ThemeText);
        check(view.colors==1 && view.getCurrentTextColor()==0xff778899 && view.getTextSize()==20);
        view.setTextAppearance(owner,R.style.ThemeText);
        check(view.colors==2 && view.seen.isStateful() && view.getCurrentTextColor()==0xff224466);
        view.measure(View.MeasureSpec.makeMeasureSpec(200,View.MeasureSpec.EXACTLY),View.MeasureSpec.makeMeasureSpec(100,View.MeasureSpec.EXACTLY));
        android.text.Layout layout=view.getLayout();
        view.setTextAppearance(owner,R.style.ThemeText);
        check(view.getLayout()==layout && view.colors==3);
        ColorStateList retained=view.seen;
        view.setTextAppearance(owner,R.style.SizeOnlyText);
        check(view.seen==retained && view.colors==3 && view.getTextSize()==23 && view.getLayout()==null);
        view.fail=true;
        try { view.setTextAppearance(owner,R.style.ThemeText); throw new IllegalStateException("appearance fault ignored"); }
        catch (IllegalStateException expected) { check("appearance callback failed".equals(expected.getMessage())); }
        check(view.getTextSize()==23 && view.getCurrentTextColor()==0xff224466);
        view.fail=false; view.setTextAppearance(R.style.ThemeText); System.gc();
        check(view.colors==5 && view.getCurrentTextColor()==0xff224466 && view.getTextSize()==20);
        Context missing=new Context(); missing.theme=missing.getResources().newTheme();
        try { view.setTextAppearance(missing,R.style.ThemeText); throw new IllegalStateException("unresolved appearance accepted"); }
        catch (RuntimeException expected) { check(expected.getClass()==RuntimeException.class); }
        check(view.colors==5 && view.getTextSize()==20 && view.getCurrentTextColor()==0xff224466);
        return view;
    }
    public static TypedArray cycle() {
        Context context=new Context(); context.theme.applyStyle(R.style.PaletteCycle,true);
        return context.obtainStyledAttributes(R.style.ThemeText,new int[]{android.R.attr.textColor});
    }
}
