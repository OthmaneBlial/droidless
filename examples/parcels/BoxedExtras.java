package org.droidless.parcels;

import android.os.Bundle;
import android.os.Parcel;
import java.io.Serializable;
import java.util.ArrayList;

/** Android's value tags carry boxed primitives; custom serializables stay inside this runtime. */
public class BoxedExtras {
    static void require(boolean value) { if (!value) throw new IllegalStateException("boxed extras contract"); }
    static Bundle copy(Bundle value) {
        Parcel parcel = Parcel.obtain();
        try { parcel.writeBundle(value); parcel.setDataPosition(0); return parcel.readBundle(); }
        finally { parcel.recycle(); }
    }
    public static int contract() {
        Integer integer = Integer.valueOf(321);
        Long wide = Long.valueOf(0x1122334455667788L);
        Double decimal = Double.valueOf(-123.5);
        Boolean bool = Boolean.valueOf(true);
        require(integer instanceof Serializable && wide instanceof Serializable && decimal instanceof Serializable && bool instanceof Serializable);
        Bundle original = new Bundle();
        original.putSerializable("int",integer); original.putSerializable("long",wide);
        original.putSerializable("double",decimal); original.putSerializable("bool",bool);
        original.putSerializable("null",null);
        require(original.getSerializable("int") == integer && original.getInt("int") == 321);
        require(original.getLong("long") == wide.longValue() && original.getDouble("double") == -123.5 && original.getBoolean("bool"));
        require(original.getInt("long",19) == 19 && original.getString("int") == null);
        Bundle shallow = new Bundle(original);
        require(shallow.getSerializable("int") == integer);
        Bundle transported = copy(original); original.clear(); System.gc();
        Serializable result = transported.getSerializable("int");
        require(result instanceof Integer && ((Integer)result).intValue() == 321);
        require(transported.getSerializable("int") == result && transported.getInt("int") == 321);
        require(((Long)transported.getSerializable("long")).longValue() == 0x1122334455667788L);
        require(((Double)transported.getSerializable("double")).doubleValue() == -123.5);
        require(((Boolean)transported.getSerializable("bool")).booleanValue());
        require(transported.containsKey("null") && transported.getSerializable("null") == null);
        transported.putInt("plain",-129);
        Bundle primitiveCopy = new Bundle(transported);
        Serializable plain = transported.getSerializable("plain");
        require(plain instanceof Integer && ((Integer)plain).intValue() == -129 && transported.getSerializable("plain") == plain && transported.getInt("plain") == -129);
        require(primitiveCopy.getSerializable("plain") == plain);
        ArrayList<Serializable> list = new ArrayList<Serializable>();
        list.add(integer); list.add(wide); list.add(decimal); list.add(bool); list.add("text"); list.add(null);
        transported.putSerializable("list",list);
        Bundle nested = copy(transported); transported.clear(); System.gc();
        ArrayList<?> values = (ArrayList<?>)nested.getSerializable("list");
        require(values.size() == 6 && ((Integer)values.get(0)).intValue() == 321 && ((Long)values.get(1)).longValue() == 0x1122334455667788L);
        require(((Double)values.get(2)).doubleValue() == -123.5 && ((Boolean)values.get(3)).booleanValue() && "text".equals(values.get(4)) && values.get(5) == null);
        return 1;
    }
    static class Custom implements Serializable { int value = 9; }
    public static int sameRuntimeCustom() {
        Custom original = new Custom();
        Bundle value = new Bundle(); value.putSerializable("custom",original);
        Custom restored = (Custom)copy(value).getSerializable("custom");
        require(restored == original && restored.value == 9);
        return 1;
    }
}
