package org.droidless.images;
import android.app.Activity;
import android.content.res.Resources;
import android.content.res.TypedArray;
import android.content.res.XmlResourceParser;
import android.util.TypedValue;

/** Real typed output, reference resolution, missing values and immutable theme snapshots. */
public final class TypedValueContract {
    static void check(boolean value) { if (!value) throw new IllegalStateException("typed value output"); }
    public static TypedArray run(Activity activity) throws Exception {
        XmlResourceParser parser=activity.getResources().getXml(R.xml.typed_values); parser.next();
        TypedArray array=activity.getResources().obtainAttributes(parser,new int[]{android.R.attr.alpha,android.R.attr.label,
            android.R.attr.width,android.R.attr.enabled,android.R.attr.textColor,android.R.attr.src,
            android.R.attr.hint,android.R.attr.orientation,android.R.attr.theme});
        TypedValue value=new TypedValue();
        check(array.getValue(0,value) && value.type==TypedValue.TYPE_FLOAT && value.getFloat()==0.75f && value.string==null);
        check(array.getValue(1,value) && value.type==TypedValue.TYPE_STRING && value.string.toString().equals("Typed output"));
        System.gc(); check(value.string.toString().equals("Typed output"));
        check(array.getValue(2,value) && value.type==TypedValue.TYPE_DIMENSION && (value.data & 15)==TypedValue.COMPLEX_UNIT_DIP);
        check(array.getValue(3,value) && value.type==TypedValue.TYPE_INT_BOOLEAN && value.data==0 && value.string==null);
        check(array.getValue(4,value) && value.data==0xff203040 && value.resourceId==0);
        check(array.getValue(5,value) && value.type==TypedValue.TYPE_STRING && value.resourceId==R.drawable.sample
            && value.string.toString().equals("res/drawable/sample.png"));
        CharSequence prior=value.string; int data=value.data;
        check(!array.getValue(6,value) && !array.getValue(7,value) && value.string==prior && value.data==data);
        check(array.getValue(8,value) && value.type==TypedValue.TYPE_REFERENCE && value.resourceId==R.style.DefaultsTheme
            && value.data==R.style.DefaultsTheme && value.string==null);
        Resources.Theme theme=activity.getResources().newTheme(); theme.applyStyle(R.style.PaletteTheme,true);
        TypedArray old=theme.obtainStyledAttributes(new int[]{R.attr.testPaletteAlias});
        theme.applyStyle(R.style.PaletteAlternate,true);
        check(old.getValue(0,value) && value.type==TypedValue.TYPE_STRING && value.resourceId==R.color.style_states
            && value.string.toString().equals("res/color/style_states.xml"));
        old.recycle(); parser.close(); System.gc();
        check(array.getValue(0,value) && value.getFloat()==0.75f);
        return array;
    }
}
