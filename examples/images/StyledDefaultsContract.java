package org.droidless.images;
import android.app.Activity;
import android.content.res.Resources;
import android.content.res.TypedArray;
import android.content.res.XmlResourceParser;
import org.xmlpull.v1.XmlPullParser;

/** Compiled default-style/explicit-style precedence, real theme dispatch and snapshot checks. */
public final class StyledDefaultsContract {
    public static void frameworkColors(Activity activity) {
        android.content.res.Resources.Theme theme=activity.getResources().newTheme();
        int[] attrs={android.R.attr.textColorPrimary,android.R.attr.textColorSecondary,android.R.attr.textColorHint,android.R.attr.disabledAlpha,android.R.attr.actionBarSize};
        theme.applyStyle(R.style.FrameworkMaterialLight_Child,true);
        TypedArray light=theme.obtainStyledAttributes(attrs);
        check(light.getColor(0,0)==0xde000000 && light.getColor(1,0)==0x8a000000
            && light.getColor(2,0)==0x80000000 && light.getFloat(3,0)==0.26f
            && light.getDimensionPixelSize(4,0)==56,"light Material defaults");
        theme.applyStyle(R.style.FrameworkMaterialDark,true); System.gc();
        TypedArray dark=theme.obtainStyledAttributes(attrs);
        check(dark.getColor(0,0)==0xffffffff && dark.getColor(1,0)==0xb3ffffff
            && dark.getFloat(3,0)==0.30f && light.getColor(1,0)==0x8a000000,"dark defaults and snapshot");
        theme.applyStyle(R.style.FrameworkMaterialOverride,true);
        check(theme.obtainStyledAttributes(attrs).getColor(1,0)==0xff123456,"application color override");
        theme.applyStyle(R.style.FrameworkMaterialAlias,true);
        TypedArray aliases=theme.obtainStyledAttributes(new int[]{android.R.attr.disabledAlpha,android.R.attr.minWidth});
        check(aliases.getFloat(0,0)==0.4f && aliases.getDimension(1,0)==31
            && aliases.getDimensionPixelSize(1,0)==31 && aliases.getDimensionPixelOffset(1,0)==31
            && aliases.getLayoutDimension(1,0)==31 && aliases.getLayoutDimension(0,17)==17
            && light.getFloat(3,0)==0.26f,"numeric theme aliases and snapshot");
        TypedArray sizes=theme.obtainStyledAttributes(R.style.FrameworkMaterialSizes,new int[]{android.R.attr.minWidth,android.R.attr.minHeight,android.R.attr.layout_width,android.R.attr.layout_height});
        check(sizes.getDimensionPixelSize(0,0)==1 && sizes.getDimensionPixelOffset(0,0)==0
            && sizes.getLayoutDimension(0,0)==1 && sizes.getDimensionPixelSize(1,0)==-1
            && sizes.getLayoutDimension(2,0)==-1 && sizes.getLayoutDimension(3,0)==-2,"pixel sizes and layout flags");
        sizes.recycle();
        aliases.recycle();
        light.recycle(); dark.recycle();
    }
    static final int[] ATTRS = {android.R.attr.label, android.R.attr.textSize, android.R.attr.textColor, android.R.attr.layout};
    static class Owner extends Activity {
        Resources.Theme theme;
        int calls; boolean fail;
        Owner(int style) { theme = getResources().newTheme(); theme.applyStyle(style, true); }
        public Resources.Theme getTheme() {
            calls++; System.gc();
            if (fail) throw new IllegalStateException("default theme fault");
            return theme;
        }
    }
    static void check(boolean value, String reason) { if (!value) throw new IllegalStateException(reason); }
    static void selected(TypedArray a) {
        check("Selected".equals(a.getString(0)), "selected label");
        check(a.getDimension(1, 0) == 13, "fallback style must not merge into selected style");
        check(a.getColor(2, 0) == 0xff303030, "selected color");
        check(a.getResourceId(3, 0) == R.layout.factory_leaf, "selected layout");
    }
    public static TypedArray run() throws Exception {
        Owner owner = new Owner(R.style.DefaultsTheme);
        TypedArray old = owner.obtainStyledAttributes(null, ATTRS, R.attr.defaultWidgetAlias, R.style.DefaultsFallback);
        check(owner.calls == 1, "real getTheme callback"); selected(old);
        selected(owner.theme.obtainStyledAttributes(null, ATTRS, R.attr.defaultWidgetStyle, R.style.DefaultsFallback));
        TypedArray fallback = owner.obtainStyledAttributes(null, ATTRS, 0, R.style.DefaultsFallback);
        check("Fallback".equals(fallback.getString(0)) && fallback.getDimension(1, 0) == 17, "explicit fallback");
        Owner missing = new Owner(R.style.DefaultsMissing);
        check(missing.obtainStyledAttributes(null, ATTRS, R.attr.defaultWidgetStyle, R.style.DefaultsFallback).getColor(2, 0) == 0xff202020, "missing default attribute");
        Owner cleared = new Owner(R.style.DefaultsNull);
        TypedArray noDefault = cleared.obtainStyledAttributes(null, ATTRS, R.attr.defaultWidgetStyle, R.style.DefaultsFallback);
        check("Theme value".equals(noDefault.getString(0)) && noDefault.getColor(2, 0) == 0xff101010, "null default suppresses fallback");
        XmlResourceParser parser = owner.getResources().getXml(R.xml.styled_defaults);
        check(parser.next() == XmlPullParser.START_TAG, "XML root");
        TypedArray xml = owner.obtainStyledAttributes(parser, ATTRS, R.attr.defaultWidgetStyle, R.style.DefaultsFallback);
        check("XML style".equals(xml.getString(0)) && xml.getDimension(1, 0) == 19 && xml.getColor(2, 0) == 0xff505050 && xml.getResourceId(3, 0) == R.layout.factory_include, "XML and style precedence");
        TypedArray plain = owner.getResources().obtainAttributes(parser, ATTRS);
        check(plain.getString(0) == null && plain.getColor(2, 0) == 0xff505050 && plain.getResourceId(3, 23) == 23, "unstyled resources");
        check(parser.nextTag() == XmlPullParser.START_TAG, "alias element");
        TypedArray alias = owner.theme.obtainStyledAttributes(parser, ATTRS, 0, R.style.DefaultsFallback);
        check("Selected".equals(alias.getString(0)) && alias.getDimension(1, 0) == 17 && alias.getColor(2, 0) == 0xff303030, "XML style retains lower default properties");
        check(parser.nextTag() == XmlPullParser.END_TAG && parser.nextTag() == XmlPullParser.START_TAG, "clear element");
        TypedArray clear = owner.obtainStyledAttributes(parser, ATTRS, R.attr.defaultWidgetStyle, 0);
        check(clear.getColor(2, 29) == 29, "explicit null wins");
        parser.close();
        owner.theme.applyStyle(R.style.DefaultsFallback, true); System.gc();
        selected(old);
        owner.fail = true;
        try { owner.obtainStyledAttributes(null, ATTRS, R.attr.defaultWidgetStyle, 0); throw new IllegalStateException("theme fault ignored"); }
        catch (IllegalStateException expected) { check("default theme fault".equals(expected.getMessage()), "theme error"); }
        owner.fail = false;
        check(owner.obtainStyledAttributes(null, ATTRS, 0, 0).getColor(2, 0) == 0xff202020, "recovery");
        return old;
    }
    public static TypedArray cycle() {
        return new Owner(R.style.DefaultsCycle).obtainStyledAttributes(null, ATTRS, R.attr.defaultWidgetStyle, R.style.DefaultsFallback);
    }
}
