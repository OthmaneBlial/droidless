package org.droidless.scheduling;

import android.os.Handler;
import android.os.Looper;
import android.os.Message;
import android.os.SystemClock;
import java.util.concurrent.LinkedBlockingQueue;

/** Compiled Android calls exercise worker delivery; no host callback substitutes. */
public class WorkerLooperContract {
    static Thread worker, peer;
    static Handler handler, peerHandler, mainHandler;
    static LinkedBlockingQueue<String> input;
    static String events, mainEvents, peerEvents, output;
    static int stage, finished, validations, caught;
    static Runnable onResult;
    static void identity() {
        if (Thread.currentThread() != worker || Looper.myLooper() != handler.getLooper())
            throw new IllegalStateException("worker Looper identity");
    }
    static class Append implements Runnable {
        final String value;
        Append(String value) { this.value = value; }
        public void run() { identity(); System.gc(); events += value; }
    }
    static class RecordingHandler extends Handler {
        RecordingHandler() { super(new Handler.Callback() {
            public boolean handleMessage(Message message) {
                identity(); events += "C" + message.what;
                return message.what == 9;
            }
        }); }
        public void handleMessage(Message message) {
            identity();
            if (message.what == 7) {
                if (message.arg1 != 11 || message.arg2 != 12 || !"payload".equals(message.obj))
                    throw new IllegalStateException("worker message fields");
                events += "H";
            } else if (message.what == 10) {
                String kept = (String) message.obj;
                stage = 2; System.gc();
                try { output = kept + ":" + input.take(); }
                catch (InterruptedException failure) { throw new IllegalStateException("callback interrupted", failure); }
                identity(); events += "B"; stage = 3;
                mainHandler.post(new Runnable() { public void run() {
                    if (Thread.currentThread() != Looper.getMainLooper().getThread())
                        throw new IllegalStateException("worker result off main");
                    mainEvents += "R";
                    if (onResult != null) onResult.run();
                }});
            }
        }
    }
    static class OverridingHandler extends RecordingHandler {
        public void dispatchMessage(Message message) { identity(); events += "D"; super.dispatchMessage(message); }
    }
    public static void prepare(final int mode, Runnable result) {
        events = mainEvents = peerEvents = output = "";
        stage = finished = validations = caught = 0; peer = null; peerHandler = null;
        input = new LinkedBlockingQueue<String>(); onResult = result;
        mainHandler = new Handler(Looper.getMainLooper());
        worker = new Thread(new Runnable() { public void run() {
            try { new Handler(); } catch (RuntimeException expected) { validations |= 1; }
            try { Looper.loop(); } catch (RuntimeException expected) { validations |= 2; }
            Looper.prepare();
            try { Looper.prepare(); } catch (RuntimeException expected) { validations |= 4; }
            handler = mode == 1 ? new OverridingHandler() : new RecordingHandler();
            identity();
            if (handler.getLooper().getThread() != worker) throw new IllegalStateException("Looper owner");
            stage = 1;
            for (;;) {
                try { Looper.loop(); break; }
                catch (IllegalStateException failure) {
                    if (mode != 2 || caught != 0 || !"looper callback failure".equals(failure.getMessage())) throw failure;
                    caught++; handler.post(new Append("E"));
                    handler.post(new Runnable() { public void run() { Looper.myLooper().quit(); }});
                }
            }
            finished++;
        }}, "looper-worker");
        worker.start();
    }
    public static void preparePeer() {
        peer = new Thread(new Runnable() { public void run() {
            Looper.prepare(); peerHandler = new Handler(); Looper.loop();
        }}, "looper-peer"); peer.start();
    }
    public static void enqueueOrdered() {
        long when = SystemClock.uptimeMillis() + 10;
        handler.postAtTime(new Append("A"), when);
        handler.sendMessageAtTime(handler.obtainMessage(9), when);
        Message fields = handler.obtainMessage(7); fields.arg1 = 11; fields.arg2 = 12; fields.obj = "payload";
        handler.sendMessageAtTime(fields, when);
        handler.postAtTime(new Append("Z"), when + 1);
        Append removed = new Append("BAD"); handler.postAtTime(removed, when); handler.removeCallbacks(removed);
        Object token = new Object(); handler.postAtTime(new Append("BAD"), token, when); handler.removeCallbacksAndMessages(token);
        mainHandler.post(new Runnable() { public void run() { mainEvents += "M"; }});
    }
    public static void enqueuePeer() {
        peerHandler.post(new Runnable() { public void run() {
            if (Thread.currentThread() != peer || Looper.myLooper() != peerHandler.getLooper())
                throw new IllegalStateException("peer Looper identity");
            peerEvents += "P";
        }});
    }
    public static void enqueueNativeNoops() {
        for (int i = 0; i < 128; i++) peerHandler.sendMessage(peerHandler.obtainMessage());
        enqueuePeer();
        mainHandler.post(new Runnable() { public void run() { mainEvents += "N"; }});
    }
    public static void enqueueBlocking() {
        Message message = handler.obtainMessage(10); message.obj = new StringBuilder().append("kept-payload").toString();
        handler.sendMessage(message); handler.post(new Append("F"));
    }
    public static void enqueueFault() {
        handler.post(new Runnable() { public void run() { throw new IllegalStateException("looper callback failure"); }});
        handler.post(new Append("R"));
    }
    public static void enqueueQuit() {
        handler.post(new Append("Q")); handler.postDelayed(new Append("BAD"), 100);
    }
    public static void enqueueChangedTarget() {
        Message message = handler.obtainMessage(9);
        handler.sendMessage(message); message.setTarget(new RecordingHandler());
    }
    public static void feed() { input.offer("input"); }
    public static void interrupt() { worker.interrupt(); }
    public static void quit(int safely) {
        if (safely != 0) handler.getLooper().quitSafely(); else handler.getLooper().quit();
    }
    public static void quitPeer() { peerHandler.getLooper().quit(); }
    public static int postRejected() { return handler.post(new Append("BAD")) ? 0 : 1; }
    public static int state() { return stage; }
    public static int alive() { return worker.isAlive() ? 1 : 0; }
    public static int finished() { return finished; }
    public static int validations() { return validations; }
    public static int caught() { return caught; }
    public static String readEvents() { return events; }
    public static String readMainEvents() { return mainEvents; }
    public static String readPeerEvents() { return peerEvents; }
    public static String readOutput() { return output; }
}
