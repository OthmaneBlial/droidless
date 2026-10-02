package org.droidless.images;

import android.app.Activity;
import android.app.Dialog;
import android.content.Context;
import android.content.DialogInterface;
import android.content.res.Resources;
import android.content.res.TypedArray;
import android.os.Bundle;
import android.util.AttributeSet;
import android.util.TypedValue;
import android.view.LayoutInflater;
import android.view.View;
import android.view.ViewGroup;
import android.view.Window;
import android.view.WindowManager;
import android.widget.FrameLayout;
import android.widget.TextView;

/** Independent dialog/window ownership, real callbacks and resource-theme resolution. */
public final class DialogWindowContract {
    static class EmptyText extends TextView {
        EmptyText(Context context) { super(context); }
        static int[] states() { return EMPTY_STATE_SET; }
    }
    static class EmptyFrame extends FrameLayout {
        EmptyFrame(Context context) { super(context); }
        static int[] states() { return EMPTY_STATE_SET; }
    }
    static class Shadow extends TextView {
        static final int[] EMPTY_STATE_SET={17};
        Shadow(Context context) { super(context); }
        static int[] states() { return EMPTY_STATE_SET; }
    }
    static class Base extends StyleColorContract.Context {
        boolean serviceFault;
        public Object getSystemService(String name) {
            System.gc();
            if (serviceFault) throw new IllegalStateException("window service failed");
            return super.getSystemService(name);
        }
    }
    static class Probe extends Dialog {
        int creates, contents, attributes;
        boolean createFault, contentFault, attributeFault, recurse;
        WindowManager.LayoutParams seen;
        Probe(Context context,int theme) { super(context,theme); }
        Probe(Context context) { super(context); }
        protected void onCreate(Bundle state) {
            creates++; System.gc(); check(state==null,"creation state");
            if (createFault) throw new IllegalStateException("dialog create failed");
            super.onCreate(state); setContentView(R.layout.factory_leaf);
        }
        public void onContentChanged() {
            contents++; System.gc();
            if (contentFault) throw new IllegalStateException("dialog content failed");
            super.onContentChanged();
        }
        public void onWindowAttributesChanged(WindowManager.LayoutParams params) {
            attributes++; seen=params; System.gc();
            if (attributeFault) throw new IllegalStateException("window attributes failed");
            check(params==getWindow().getAttributes(),"attribute callback identity");
            if (recurse) getWindow().setLayout(271,173);
            super.onWindowAttributesChanged(params);
        }
    }
    static class Factory implements LayoutInflater.Factory {
        Context seen;
        public View onCreateView(String name,Context context,AttributeSet attrs) {
            seen=context; System.gc(); return null;
        }
    }
    static void check(boolean value,String reason) { if (!value) throw new IllegalStateException(reason); }
    static int color(Context context) {
        TypedArray a=context.obtainStyledAttributes(R.style.ThemeText,new int[]{android.R.attr.textColor});
        int color=a.getColor(0,0); a.recycle(); return color;
    }
    public static void recursive(Dialog dialog,boolean enabled) { ((Probe)dialog).recurse=enabled; }
    public static Dialog run(Activity activity) {
        int[] empty=EmptyText.states(); System.gc();
        check(empty.length==0 && empty==EmptyFrame.states() && Shadow.states().length==1
            && Shadow.states()[0]==17 && Shadow.states()!=empty,"native inherited empty states and guest shadow");
        Window main=activity.getWindow(); View mainRoot=main.getDecorView();
        check(main==activity.getWindow() && main.getContext()==activity && main.getCallback()==activity,"Activity window identity");
        Base base=new Base(); base.theme.applyStyle(R.style.DialogHostTheme,true);
        TypedValue typed=new TypedValue(); typed.data=123;
        check(!base.theme.resolveAttribute(R.attr.testPaletteAlias+10000,typed,true) && typed.data==123,"missing attribute leaves value unchanged");
        check(base.theme.resolveAttribute(android.R.attr.dialogTheme,typed,false)
            && typed.type==TypedValue.TYPE_REFERENCE && typed.data==R.style.PaletteAlternate && typed.resourceId==0,"unresolved style reference");
        check(base.theme.resolveAttribute(android.R.attr.dialogTheme,typed,true)
            && typed.type==TypedValue.TYPE_REFERENCE && typed.resourceId==R.style.PaletteAlternate,"resolved style reference");
        check(base.theme.resolveAttribute(R.attr.testPaletteAlias,typed,true)
            && typed.type==TypedValue.TYPE_STRING && typed.resourceId==R.color.style_states && typed.string!=null,"theme/resource aliases");
        base.theme.applyStyle(R.style.PaletteCycle,true); typed.data=345;
        check(!base.theme.resolveAttribute(R.attr.testPaletteAlias,typed,true) && typed.data==345,"attribute cycle leaves value unchanged");
        base.theme.applyStyle(R.style.PaletteTheme,true);
        Probe dialog=new Probe(base);
        check(dialog instanceof DialogInterface && !dialog.isShowing() && dialog.creates==0,"dialog construction/lifecycle");
        check(dialog.getContext()!=base && color(dialog.getContext())==0xff778899 && color(base)==0xff224466,"default dialog theme/independent context");
        Window window=dialog.getWindow();
        check(window!=main && window==dialog.getWindow() && window.getContext()==dialog.getContext()
            && window.getCallback()==dialog && window.peekDecorView()==null,"independent Window/callback/decor");
        check(window.getWindowManager()==base.getSystemService(Context.WINDOW_SERVICE),"virtual window service");
        check(dialog.getOwnerActivity()==null,"no implicit owner Activity");
        dialog.setOwnerActivity(activity); check(dialog.getOwnerActivity()==activity,"explicit owner Activity");
        check(dialog.findViewById(android.R.id.content)==null && dialog.getCurrentFocus()==null,"uncreated content/focus");
        Factory factory=new Factory(); LayoutInflater.from(base).setFactory(factory);
        LayoutInflater inflater=dialog.getLayoutInflater();
        check(inflater==window.getLayoutInflater() && inflater.getContext()==dialog.getContext() && inflater.getFactory()==factory,"window inflater/context/factory");
        dialog.create(); dialog.create();
        check(dialog.creates==1 && dialog.contents==1 && !dialog.isShowing() && factory.seen==dialog.getContext(),"single real creation and content callback");
        View decor=window.getDecorView();
        ViewGroup container=(ViewGroup)dialog.findViewById(android.R.id.content);
        View first=container.getChildAt(0);
        check(decor!=mainRoot && window.peekDecorView()==decor && first.getParent()==container,"separate stable decor/container");
        TextView replacement=new TextView(dialog.getContext()); replacement.setId(1234);
        dialog.setContentView(replacement,new FrameLayout.LayoutParams(31,37));
        check(window.getDecorView()==decor && first.getParent()==null && replacement.getParent()==container
            && dialog.findViewById(1234)==replacement && replacement.getLayoutParams().width==31,"content replacement/parameters");
        check(replacement.requestFocus()==false,"text label is not focusable");
        TextView extra=new TextView(dialog.getContext());
        dialog.addContentView(extra,new FrameLayout.LayoutParams(21,24));
        check(container.getChildCount()==2 && extra.getParent()==container && dialog.contents==3,"add content");
        check(!decor.isAttachedToWindow() && decor.getWindowToken()==null && main.getDecorView()==mainRoot,"unshown dialog leaves Activity/attachment unchanged");
        WindowManager.LayoutParams attrs=window.getAttributes(); int oldAttributes=dialog.attributes;
        window.setLayout(301,211); window.setGravity(3); window.setFlags(0x12,0xff); window.addFlags(0x20); window.clearFlags(2);
        check(window.getAttributes()==attrs && attrs.width==301 && attrs.height==211 && attrs.gravity==3 && attrs.flags==0x30
            && dialog.attributes==oldAttributes+5 && dialog.seen==attrs,"retained attributes and callbacks");
        WindowManager.LayoutParams other=main.getAttributes(); other.width=181; other.height=191; other.flags=4;
        window.setAttributes(other); other.width=333;
        check(window.getAttributes()==attrs && attrs.width==181 && attrs.flags==4 && attrs!=other,"attribute copy/identity");
        dialog.setTitle("Dialog title"); check("Dialog title".equals(attrs.getTitle()),"window title state");
        check(!window.hasFeature(Window.FEATURE_NO_TITLE) && dialog.requestWindowFeature(Window.FEATURE_NO_TITLE)
            && window.hasFeature(Window.FEATURE_NO_TITLE) && !main.hasFeature(Window.FEATURE_NO_TITLE),"independent features");
        dialog.contentFault=true;
        try { dialog.setContentView(first); throw new AssertionError("content fault ignored"); }
        catch (IllegalStateException expected) { check("dialog content failed".equals(expected.getMessage()),"content fault"); }
        dialog.contentFault=false; dialog.setContentView(replacement);
        dialog.attributeFault=true;
        try { window.setLayout(241,161); throw new AssertionError("attribute fault ignored"); }
        catch (IllegalStateException expected) { check("window attributes failed".equals(expected.getMessage()),"attribute fault"); }
        check(attrs.width==241 && attrs.height==161,"attributes committed before callback fault");
        dialog.attributeFault=false; window.setLayout(242,162);
        Probe broken=new Probe(base,R.style.PaletteTheme); broken.createFault=true;
        try { broken.create(); throw new AssertionError("creation fault ignored"); }
        catch (IllegalStateException expected) { check("dialog create failed".equals(expected.getMessage()),"create fault"); }
        broken.createFault=false; broken.create(); check(broken.creates==2 && !broken.isShowing(),"creation retry");
        base.serviceFault=true;
        try { new Probe(base,R.style.PaletteTheme); throw new AssertionError("service fault ignored"); }
        catch (IllegalStateException expected) { check("window service failed".equals(expected.getMessage()),"service fault"); }
        base.serviceFault=false; check(new Probe(base,R.style.PaletteTheme).getWindow()!=null,"constructor retry");
        System.gc(); return dialog;
    }
}
