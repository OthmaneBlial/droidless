package org.droidless.images;
import android.app.Activity;
import android.app.Dialog;
import android.content.DialogInterface;
import android.os.Handler;
import android.os.Message;
import android.view.KeyEvent;

/** Real queued callbacks, weak owners, copied messages and unshown cancellation. */
public final class DialogListenerContract {
    static class Recorder implements DialogInterface.OnCancelListener, DialogInterface.OnDismissListener,
            DialogInterface.OnShowListener, DialogInterface.OnKeyListener {
        int cancels,dismisses,shows;
        boolean nullOwner,fail;
        public void onCancel(DialogInterface owner) {
            System.gc(); cancels++; nullOwner=owner==null;
            if (fail) throw new IllegalStateException("cancel callback failed");
        }
        public void onDismiss(DialogInterface owner) { System.gc(); dismisses++; }
        public void onShow(DialogInterface owner) { System.gc(); shows++; }
        public boolean onKey(DialogInterface owner,int code,KeyEvent event) { return true; }
    }
    static class Probe extends Dialog {
        int dismisses;
        Probe(Activity activity) { super(activity,R.style.PaletteTheme); }
        public void dismiss() { dismisses++; super.dismiss(); }
    }
    static class Messages extends Handler {
        Object original;
        int calls;
        public void handleMessage(Message message) {
            System.gc(); calls++;
            check((message.what==42 && "changed".equals(message.obj)) ||
                  (message.what==7 && message.arg1==8 && message.arg2==9 && message.obj==original),"message snapshot");
        }
    }
    static Probe dialog,other,fault;
    static Recorder first,second,orphan,failing;
    static Messages messages;
    static void check(boolean okay,String message) { if(!okay) throw new IllegalStateException(message); }
    public static Dialog prepare(Activity activity) {
        dialog=new Probe(activity); first=new Recorder(); second=new Recorder();
        dialog.setOnCancelListener(first);dialog.setOnDismissListener(first);
        dialog.setOnShowListener(first);dialog.setOnKeyListener(first);
        dialog.setCancelable(false);dialog.onBackPressed();
        dialog.setCanceledOnTouchOutside(false);dialog.onBackPressed();
        check(dialog.dismisses==0,"noncancelable Back");
        dialog.setCanceledOnTouchOutside(true);dialog.onBackPressed();
        dialog.setOnCancelListener(second);dialog.cancel();
        check(dialog.dismisses==2 && first.cancels==0 && second.cancels==0 && !dialog.isShowing(),"queued single cancellation");
        other=new Probe(activity);other.setOnCancelListener(second);
        other.setCancelable(true);other.setCanceledOnTouchOutside(false);other.onBackPressed();
        messages=new Messages();messages.original=new Object();
        Message template=messages.obtainMessage(7,messages.original);template.arg1=8;template.arg2=9;
        check(messages.sendMessage(template),"template send");
        Message copy=Message.obtain(template);check(copy!=template && copy.getTarget()==messages && copy.getWhen()==0,"copied target/time");
        template.what=42;template.obj="changed";template.arg1=99;
        copy.sendToTarget();
        try { template.sendToTarget();throw new AssertionError("double send accepted"); }
        catch(IllegalStateException expected) { }
        System.gc();return dialog;
    }
    public static int delivered() {
        check(first.cancels==1 && second.cancels==1 && !first.nullOwner && !second.nullOwner,"cancel payload snapshot");
        check(first.dismisses==0 && first.shows==0 && messages.calls==2,"unshown lifecycle");return 1;
    }
    public static void orphan(Activity activity) {
        orphan=new Recorder();Probe unrooted=new Probe(activity);unrooted.setOnCancelListener(orphan);unrooted.cancel();
    }
    public static int orphanDelivered() { check(orphan.cancels==1 && orphan.nullOwner,"weak dialog owner");return 1; }
    public static Dialog fault(Activity activity) {
        fault=new Probe(activity);failing=new Recorder();failing.fail=true;
        fault.setOnCancelListener(failing);fault.cancel();return fault;
    }
    public static void recover(Activity activity) { fault=null;orphan(activity); }
    public static void custom(Activity activity) {
        Probe value=new Probe(activity);Message template=messages.obtainMessage(7,messages.original);template.arg1=8;template.arg2=9;
        value.setCancelMessage(template);value.setDismissMessage(null);value.cancel();template.what=42;template.obj="changed";
    }
    public static int customDelivered() { check(messages.calls==3,"custom cancellation message");return 1; }
    public static void drop() {dialog=null;other=null;fault=null;first=null;second=null;orphan=null;failing=null;messages=null;}
}
