package org.droidless.parcels;

import android.app.Activity;
import android.content.Intent;
import android.net.Uri;
import android.os.Bundle;
import android.os.Parcel;
import android.os.Parcelable;
import android.widget.Button;
import android.widget.LinearLayout;
import android.widget.TextView;
import java.util.ArrayList;

/** Authored contract fixture. Public APK compatibility is checked separately. */
public class MainActivity extends Activity {
    static ArrayList<Item> original;
    static Item source;
    static boolean mutate;
    static int writes, reads;
    static MainActivity home;
    TextView status;

    static void check(boolean value) { if (!value) throw new IllegalStateException("parcel contract failed"); }
    public static class Item implements Parcelable {
        int number;
        String text;
        Uri uri;
        Item(int n) { number=n; text="image é 🚀 " + n; uri=Uri.parse("content://example/image/" + n); }
        Item(Parcel in) {
            System.gc(); reads++;
            number=in.readInt(); text=in.readString(); uri=in.readParcelable(Uri.class.getClassLoader());
        }
        public void writeToParcel(Parcel out, int flags) {
            writes++;
            out.writeInt(number); out.writeString(text); out.writeParcelable(uri, flags);
            if (mutate) { original.clear(); System.gc(); }
        }
        public int describeContents() { return 0; }
        public static final Creator<Item> CREATOR = new Creator<Item>() {
            public Item createFromParcel(Parcel in) { return new Item(in); }
            public Item[] newArray(int size) { return new Item[size]; }
        };
    }
    public static class Broken implements Parcelable {
        public void writeToParcel(Parcel out, int flags) { System.gc(); throw new IllegalStateException("writer failed"); }
        public int describeContents() { return 0; }
        public static final Creator<Broken> CREATOR = new Creator<Broken>() {
            public Broken createFromParcel(Parcel in) { throw new IllegalStateException("creator failed"); }
            public Broken[] newArray(int size) { return new Broken[size]; }
        };
    }
    public static int contract() {
        check(Item.class.getClassLoader() != null && Uri.class.getClassLoader() == null);
        check(Item.class.getClassLoader() == MainActivity.class.getClassLoader());
        Parcel p=Parcel.obtain();
        p.writeInt(0); p.writeLong(0x1122334455667788L); p.writeFloat(1.25f); p.writeDouble(-3.5);
        p.writeString(null); p.writeString("é 🚀"); p.writeParcelable(null, 0);
        Item source=new Item(7); p.writeParcelable(source, 0);
        int end=p.dataPosition(); check(end == p.dataSize() && p.dataAvail() == 0);
        p.setDataPosition(0); p.writeInt(42); check(p.dataSize() == end);
        p.setDataPosition(0);
        check(p.readInt()==42 && p.readLong()==0x1122334455667788L && p.readFloat()==1.25f && p.readDouble()==-3.5);
        check(p.readString()==null && "é 🚀".equals(p.readString()) && p.readParcelable(null)==null);
        Item copy=p.readParcelable(Item.class.getClassLoader());
        check(copy != source && copy.number==7 && copy.text.equals(source.text) && copy.uri != source.uri);
        check(copy.uri.toString().equals(source.uri.toString()) && p.dataAvail()==0);
        p.recycle();
        Bundle nested=new Bundle(); nested.putInt("n", 8); nested.putBoolean("b", true);
        ArrayList<Item> items=new ArrayList<Item>(); items.add(source); items.add(null); items.add(new Item(9));
        Bundle bundle=new Bundle(); bundle.putBundle("nested", nested); bundle.putParcelableArrayList("items", items);
        bundle.putParcelable("one", source); bundle.putString("null", null);
        Bundle shallow=new Bundle(bundle); check(shallow.getParcelable("one")==source);
        p=Parcel.obtain(); p.writeBundle(bundle); p.writeBundle(null); p.writeBundle(new Bundle()); p.setDataPosition(0);
        Bundle decoded=p.readBundle(Item.class.getClassLoader());
        check(decoded != bundle && decoded.getBundle("nested") != nested && decoded.getBundle("nested").getInt("n")==8);
        check(decoded.getBundle("nested").getBoolean("b") && decoded.containsKey("null") && decoded.getString("null")==null);
        ArrayList<Item> got=decoded.getParcelableArrayList("items");
        check(got != items && got.size()==3 && got.get(0) != source && got.get(1)==null && got.get(2).number==9);
        check(decoded.getParcelable("one") instanceof Item && decoded.getInt("one", 17)==17 && decoded.getParcelableArrayList("nested")==null);
        check(p.readBundle()==null && p.readBundle().isEmpty() && p.dataAvail()==0); p.recycle();
        return 1;
    }
    public void onCreate(Bundle state) {
        super.onCreate(state); home=this; setTitle("Parcel home");
        LinearLayout layout=new LinearLayout(this); layout.setOrientation(1);
        status=new TextView(this); status.setText("Ready"); layout.addView(status);
        Button open=new Button(this); open.setText("Open parcel detail");
        open.setOnClickListener(v -> open()); layout.addView(open); setContentView(layout);
    }
    void open() {
        source=new Item(70); original=new ArrayList<Item>();
        original.add(source); original.add(null); original.add(new Item(71)); original.add(new Item(72));
        Bundle nested=new Bundle(); nested.putParcelableArrayList("items", original); nested.putInt("value", 5);
        Intent intent=new Intent(this, Child.class).putExtra("state", nested);
        mutate=true; startActivityForResult(intent, 12); mutate=false;
        source.number=999; nested.putInt("value", 999); System.gc();
    }
    protected void onActivityResult(int request, int code, Intent intent) {
        check(request==12 && code==RESULT_OK);
        Item result=intent.getParcelableExtra("result");
        check(result.number==70 && result != source);
        status.setText("Parcel result 70");
    }
    public static Object failedWriter() {
        Broken broken=new Broken();
        try { home.startActivity(new Intent(home, Child.class).putExtra("broken", broken)); check(false); }
        catch (IllegalStateException expected) { check(expected.getMessage().equals("writer failed")); }
        return broken;
    }
    public static int failedCreator() {
        Parcel p=Parcel.obtain(); p.writeString(Broken.class.getName()); p.setDataPosition(0);
        try { p.readParcelable(Broken.class.getClassLoader()); check(false); }
        catch (IllegalStateException expected) { check(expected.getMessage().equals("creator failed")); }
        p.setDataPosition(0); check(p.readString().equals(Broken.class.getName())); p.recycle();
        return 1;
    }
    public static class Child extends Activity {
        Item item;
        public void onCreate(Bundle state) {
            super.onCreate(state); setTitle("Parcel detail");
            Bundle nested=getIntent().getBundleExtra("state");
            ArrayList<Item> items=nested.getParcelableArrayList("items");
            item=items.get(0);
            check(items.size()==4 && items.get(1)==null && items.get(2).number==71 && items.get(3).number==72);
            check(item != source && item.number==70 && nested.getInt("value")==5);
            TextView label=new TextView(this); label.setText("4 entries · isolated state 70"); setContentView(label);
        }
        public void onBackPressed() {
            Intent result=new Intent().putExtra("result", item);
            setResult(RESULT_OK, result); finish(); item.number=888; System.gc();
        }
    }
}
