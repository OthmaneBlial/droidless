package org.droidless.counter;

import android.app.Activity;
import android.content.Context;
import android.content.res.ColorStateList;
import android.graphics.drawable.ColorDrawable;
import android.util.AttributeSet;
import android.view.LayoutInflater;
import android.view.Menu;
import android.view.MenuInflater;
import android.view.MenuItem;
import android.view.View;
import android.widget.TextView;
import android.widget.LinearLayout;

/** Authored API checks compiled by javac and D8, not public APK evidence. */
public final class MenuContract {
    static Menu saved;
    public static class Label extends LinearLayout {
        int finishes;
        TextView child;
        public Label(Context context, AttributeSet attrs) {
            super(context, attrs); System.gc();
            check(findViewById(R.id.tag_child) == null, "children attached before constructor");
        }
        @Override protected void onFinishInflate() {
            super.onFinishInflate(); finishes++; System.gc();
            child = (TextView) findViewById(R.id.tag_child);
            check(child != null && child.getText().toString().equals("From class tag"), "finish before inflated children/attributes");
        }
    }
    static class Tint extends ColorDrawable {
        ColorStateList colors;
        @Override public void setTintList(ColorStateList value) {
            System.gc(); colors = value; super.setTintList(value);
        }
    }
    static class BadTint extends ColorDrawable {
        @Override public void setTintList(ColorStateList value) {
            System.gc(); throw new IllegalStateException("tint failed");
        }
    }
    static void check(boolean value, String why) {
        if (!value) throw new AssertionError(why);
    }
    public static int verify(Menu menu, Activity activity) {
        saved = menu;
        View inflated = LayoutInflater.from(activity).inflate(R.layout.view_tag, null);
        check(inflated instanceof Label && ((Label) inflated).finishes == 1
                && ((Label) inflated).child == inflated.findViewById(R.id.tag_child)
                && inflated.getContext() == activity, "generic view class tag constructor and attributes");
        check(inflated.getAccessibilityLiveRegion() == View.ACCESSIBILITY_LIVE_REGION_NONE, "default live region");
        for (int mode : new int[]{0, 1, 2, 3, 4, -1}) {
            inflated.setAccessibilityLiveRegion(mode); System.gc();
            check(inflated.getAccessibilityLiveRegion() == (mode & 3), "live region mode bits and GC");
        }
        check(menu.size() == 0 && !menu.hasVisibleItems(), "empty menu");
        MenuItem late = menu.add(4, 10, 8, "Late");
        MenuItem early = menu.add(4, 11, 2, "Early");
        MenuItem equal = menu.add(5, 10, 8, "Equal");
        MenuItem category = menu.add(0, 12, Menu.CATEGORY_ALTERNATIVE | 1, "Category");
        check(menu.getItem(0) == early && menu.getItem(1) == late
                && menu.getItem(2) == equal && menu.getItem(3) == category, "stable ordering");
        MenuItem top = menu.add(0, 13, (5 << 16) | 9, "Top");
        check(menu.getItem(0) == top && top.getOrder() == ((5 << 16) | 9), "category mapping");
        check(late.getGroupId() == 4 && late.getItemId() == 10
                && late.isVisible() && late.isEnabled() && !late.isChecked()
                && !late.isCheckable() && !late.hasSubMenu() && late.getSubMenu() == null, "item defaults");
        late.setTitle("Renamed").setTitleCondensed("Short");
        check(late.getTitle().equals("Renamed") && late.getTitleCondensed().equals("Short"), "titles");
        early.setTitle(R.string.app_name);
        check(early.getTitleCondensed().equals("DROIDLESS Counter fixture"), "resource title fallback");
        menu.setGroupCheckable(4, true, true);
        late.setChecked(true); early.setChecked(true);
        check(!late.isChecked() && early.isChecked(), "exclusive group");
        late.setChecked(false);
        check(late.isChecked() && !early.isChecked(), "exclusive selects even false");
        menu.setGroupCheckable(4, true, false);
        early.setChecked(true); late.setChecked(false);
        check(early.isChecked() && !late.isChecked(), "nonexclusive group");
        menu.setGroupEnabled(4, false); menu.setGroupVisible(4, false);
        check(!early.isEnabled() && !late.isVisible() && equal.isEnabled() && equal.isVisible(), "group flags");
        try { menu.getItem(-1); throw new AssertionError("negative item index"); }
        catch (IndexOutOfBoundsException expected) {}
        try { menu.add(0, 0, 6 << 16, "Bad"); throw new AssertionError("invalid category"); }
        catch (IllegalArgumentException expected) {}
        try { late.setShowAsAction(3); throw new AssertionError("conflicting action flags"); }
        catch (IllegalArgumentException expected) {}
        check(late.setShowAsActionFlags(MenuItem.SHOW_AS_ACTION_IF_ROOM) == late, "fluent action flags");
        menu.removeItem(10);
        check(menu.findItem(10) == equal, "only first duplicate removed");
        menu.removeGroup(4);
        check(menu.findItem(11) == null && menu.size() == 3, "remove group");
        menu.clear();
        MenuInflater inflater = activity.getMenuInflater();
        inflater.inflate(R.menu.contract, menu);
        check(menu.size() == 3 && menu.getItem(0).getItemId() == R.id.choice_a
                && menu.getItem(1).getItemId() == R.id.choice_b, "XML ordering");
        MenuItem a = menu.findItem(R.id.choice_a), b = menu.findItem(R.id.choice_b);
        check(a.getGroupId() == R.id.choices && a.isCheckable() && a.isChecked()
                && !a.isEnabled() && !a.isVisible() && a.getTitleCondensed().equals("A"), "XML group inheritance");
        check(b.isEnabled() && b.isVisible() && b.getTitle().equals("Literal B"), "XML item override");
        b.setChecked(true);
        check(!a.isChecked() && b.isChecked(), "XML exclusive group");
        MenuItem tail = menu.getItem(2);
        check(tail.getTitle().equals("DROIDLESS Counter fixture") && tail.getIcon() != null, "XML title and icon");
        Tint tint = new Tint();
        tint.setTint(0xff123456);
        check(tint.colors.getDefaultColor() == 0xff123456, "virtual tint dispatch");
        tail.setIcon(tint).setOnMenuItemClickListener(new MenuItem.OnMenuItemClickListener() {
            public boolean onMenuItemClick(MenuItem item) { System.gc(); return item == saved.getItem(2); }
        });
        System.gc();
        check(tail.getIcon() == tint && tint.colors.getDefaultColor() == 0xff123456, "menu and tint GC roots");
        return 1;
    }
    public static ColorDrawable failedTint() {
        ColorDrawable drawable = new BadTint();
        try { drawable.setTint(42); throw new AssertionError("tint failure swallowed"); }
        catch (IllegalStateException expected) { check(expected.getMessage().equals("tint failed"), "tint error"); }
        return drawable;
    }
    public static void clear() { saved = null; }
}
