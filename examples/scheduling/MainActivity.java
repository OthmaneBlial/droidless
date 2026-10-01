package org.droidless.scheduling;

import android.app.Activity;
import android.os.Bundle;
import android.os.Handler;
import android.os.Looper;
import android.os.Message;
import android.os.SystemClock;
import android.view.View;
import android.widget.Button;
import android.widget.LinearLayout;
import android.widget.TextView;
import java.util.Timer;
import java.util.TimerTask;
import java.util.concurrent.*;

/** Authored timer/conformance UI. Its callbacks run as DEX, not host timer logic. */
public class MainActivity extends Activity {
    public static String events = "";
    static int dispatches;
    static int messageCallbacks;
    static int handled;
    static Handler handler;
    static Message queued;
    Handler timerHandler = new Handler(Looper.getMainLooper());
    TextView label;
    int ticks;
    Timer backgroundTimer;
    ExecutorService futurePool;
    FutureTask<String> futureTask;
    LinkedBlockingQueue<String> futureInput;
    boolean looperRunning;
    Object looperOwner;
    final Runnable timer = new Runnable() {
        public void run() {
            if (Thread.currentThread() != Looper.getMainLooper().getThread()) throw new IllegalStateException("wrong timer thread");
            ticks++;
            label.setText(ticks == 3 ? "Timer done: 3" : "Tick " + ticks);
            if (ticks < 3) timerHandler.postDelayed(this, 1500);
        }
    };
    static class RecordingHandler extends Handler {
        RecordingHandler() { super(Looper.getMainLooper(), new Handler.Callback() {
            public boolean handleMessage(Message message) {
                messageCallbacks++;
                if (message.what == 9) {
                    if (message.arg1 != -11 || message.arg2 != -12 || !"payload".equals(message.obj))
                        throw new IllegalStateException("message factory arguments");
                    events += "C"; return true;
                }
                return false;
            }
        }); }
        public void dispatchMessage(Message message) { dispatches++; super.dispatchMessage(message); }
        public void handleMessage(Message message) {
            handled++;
            if (message.what != 7 || message.arg1 != 11 || message.arg2 != 12 || !"payload".equals(message.obj)) throw new IllegalStateException("message fields");
            events += "M";
            boolean rejected = false;
            try { sendMessage(message); } catch (IllegalStateException expected) { rejected = true; }
            if (!rejected) throw new IllegalStateException("queued message reused");
        }
    }
    static class Append implements Runnable {
        String value;
        Append(String value) { this.value = value; }
        public void run() { System.gc(); events += value; }
    }
    static class EqualToken {
        public boolean equals(Object other) { return other instanceof EqualToken; }
    }
    static class LayoutAnimation extends TextView {
        boolean started;
        LayoutAnimation(Activity context) { super(context); setText("Layout animation"); }
        protected void onLayout(boolean changed, int left, int top, int right, int bottom) {
            super.onLayout(changed, left, top, right, bottom);
            if (!started) {
                started = true;
                animate().translationX(100f).alpha(0f).setDuration(100).setInterpolator(null);
            }
        }
    }
    public void startUnsafeWorker() {
        WorkerContract.prepare(4, new Runnable() { public void run() { label.setText("Wrong worker UI"); } });
    }
    public void startUnsafeFuture() {
        futurePool = Executors.newSingleThreadExecutor();
        futurePool.submit(new Callable<String>() { public String call() { label.setText("Wrong Future UI"); return "wrong"; } });
    }
    public void onCreate(Bundle state) {
        super.onCreate(state);
        if (ThreadContract.contract() != 1) throw new IllegalStateException("thread metadata");
        if (Looper.myLooper() != Looper.getMainLooper() || timerHandler.getLooper() != Looper.getMainLooper()) throw new IllegalStateException("main Looper");
        boolean rejected = false;
        try { Looper.getMainLooper().quit(); } catch (IllegalStateException expected) { rejected = true; }
        if (!rejected) throw new IllegalStateException("main quit allowed");
        LinearLayout layout = new LinearLayout(this); layout.setOrientation(1); layout.setPadding(24,24,24,24);
        label = new TextView(this); label.setText("Scheduling ready"); label.setTextSize(24); layout.addView(label);
        Button start = new Button(this); start.setText("Start timer");
        start.setOnClickListener(new View.OnClickListener() { public void onClick(View view) {
            timerHandler.removeCallbacks(timer); ticks = 0; label.setText("Waiting for timer"); timerHandler.postDelayed(timer,1500);
        }}); layout.addView(start);
        Button animation = new Button(this); animation.setText("Start layout animation");
        animation.setOnClickListener(new View.OnClickListener() { public void onClick(View view) {
            setContentView(new LayoutAnimation(MainActivity.this));
        }}); layout.addView(animation);
        Button cancel = new Button(this); cancel.setText("Cancel timer");
        cancel.setOnClickListener(new View.OnClickListener() { public void onClick(View view) {
            timerHandler.removeCallbacks(timer); label.setText("Timer cancelled");
        }}); layout.addView(cancel);
        Button finish = new Button(this); finish.setText("Finish later");
        finish.setOnClickListener(new View.OnClickListener() { public void onClick(View view) {
            timerHandler.postDelayed(new Runnable() { public void run() { MainActivity.this.finish(); } },20);
        }}); layout.addView(finish);
        Button worker = new Button(this); worker.setText("Start worker");
        worker.setOnClickListener(new View.OnClickListener() { public void onClick(View view) {
            label.setText("Worker queued");
            WorkerContract.prepare(0, new Runnable() { public void run() {
                if (Looper.myLooper() != null || Thread.currentThread() == Looper.getMainLooper().getThread()) throw new IllegalStateException("worker Looper");
                Looper.prepare();
                if (Looper.myLooper().getThread() != Thread.currentThread()) throw new IllegalStateException("prepared Looper");
                boolean rejected = false;
                try { Looper.prepare(); } catch (RuntimeException expected) { rejected = true; }
                if (!rejected) throw new IllegalStateException("second Looper allowed");
                timerHandler.post(new Runnable() { public void run() {
                    if (Thread.currentThread() != Looper.getMainLooper().getThread()) throw new IllegalStateException("worker result off main");
                    label.setText("Worker result: " + WorkerContract.pollOutput());
                    WorkerContract.interrupt();
                }});
            }});
            try { WorkerContract.feed(); } catch (InterruptedException failure) { throw new IllegalStateException("main interrupted"); }
        }}); layout.addView(worker);
        Button sleepJoin = new Button(this); sleepJoin.setText("Sleep, join and finish");
        sleepJoin.setOnClickListener(new View.OnClickListener() { public void onClick(View view) {
            if (ThreadWaitContract.alive() != 0) return;
            final long started = SystemClock.uptimeMillis(); label.setText("Waiting for sleep and join");
            ThreadWaitContract.prepare(0, 0, new Runnable() { public void run() {
                if (Thread.currentThread() == Looper.getMainLooper().getThread() || SystemClock.uptimeMillis() - started < 10)
                    throw new IllegalStateException("early/main join callback");
                timerHandler.post(new Runnable() { public void run() {
                    if (Thread.currentThread() != Looper.getMainLooper().getThread()
                            || ThreadWaitContract.state() != 2 || ThreadWaitContract.joined() != 2
                            || ThreadWaitContract.contended() != 1 || ThreadWaitContract.finished() != 1
                            || ThreadWaitContract.alive() != 0 || !"kept-sleep".equals(ThreadWaitContract.output()))
                        throw new IllegalStateException("sleep/join main result");
                    label.setText("Sleep joined: kept-sleep");
                    android.util.Log.i("SleepJoin", "Worker slept, joined and delivered on main");
                    MainActivity.this.finish();
                }});
            }});
        }}); layout.addView(sleepJoin);
        Button startLooper = new Button(this); startLooper.setText("Start Looper worker");
        startLooper.setOnClickListener(new View.OnClickListener() { public void onClick(View view) {
            if (WorkerLooperContract.alive() != 0) return;
            final Object owner = new Object(); looperOwner = owner;
            looperRunning = true; label.setText("Looper waiting for input");
            WorkerLooperContract.prepare(3, new Runnable() { public void run() {
                if (looperOwner == owner && looperRunning) {
                    label.setText("Looper result: " + WorkerLooperContract.readOutput());
                    WorkerLooperContract.quit(1); looperRunning = false;
                }
            }});
        }}); layout.addView(startLooper);
        Button deliverLooper = new Button(this); deliverLooper.setText("Deliver Looper input");
        deliverLooper.setOnClickListener(new View.OnClickListener() { public void onClick(View view) {
            if (looperRunning) WorkerLooperContract.feed();
        }}); layout.addView(deliverLooper);
        Button cancelLooper = new Button(this); cancelLooper.setText("Cancel Looper worker");
        cancelLooper.setOnClickListener(new View.OnClickListener() { public void onClick(View view) {
            if (looperRunning) {
                WorkerLooperContract.quit(0); WorkerLooperContract.feed(); looperRunning = false; looperOwner = null;
            }
            label.setText("Looper worker cancelled");
        }}); layout.addView(cancelLooper);
        Button background = new Button(this); background.setText("Start background timer");
        background.setOnClickListener(new View.OnClickListener() { public void onClick(View view) {
            if (backgroundTimer != null) backgroundTimer.cancel();
            final Timer owner = new Timer("fixture-background", true); backgroundTimer = owner;
            label.setText("Background timer queued");
            owner.scheduleAtFixedRate(new TimerTask() {
                int delivered;
                public void run() {
                    if (Looper.myLooper() != null || Thread.currentThread() == Looper.getMainLooper().getThread()) throw new IllegalStateException("Timer ran on main");
                    System.gc();
                    final int tick = ++delivered;
                    if (tick == 3) owner.cancel();
                    timerHandler.post(new Runnable() { public void run() {
                        if (Thread.currentThread() != Looper.getMainLooper().getThread()) throw new IllegalStateException("Timer result off main");
                        if (backgroundTimer == owner) label.setText(tick == 3 ? "Background timer done: 3" : "Background tick " + tick);
                    }});
                }
            }, 1500, 1500);
        }}); layout.addView(background);
        Button stopBackground = new Button(this); stopBackground.setText("Cancel background timer");
        stopBackground.setOnClickListener(new View.OnClickListener() { public void onClick(View view) {
            if (backgroundTimer != null) backgroundTimer.cancel(); backgroundTimer = null;
            label.setText("Background timer cancelled");
        }}); layout.addView(stopBackground);
        Button startFuture = new Button(this); startFuture.setText("Start future worker");
        startFuture.setOnClickListener(new View.OnClickListener() { public void onClick(View view) {
            if (futureTask != null) futureTask.cancel(true);
            if (futurePool != null) futurePool.shutdownNow();
            final ExecutorService owner = Executors.newSingleThreadExecutor(); futurePool = owner;
            final LinkedBlockingQueue<String> input = new LinkedBlockingQueue<String>(); futureInput = input;
            futureTask = new FutureTask<String>(new Callable<String>() { public String call() throws Exception {
                if (Looper.myLooper() != null || Thread.currentThread() == Looper.getMainLooper().getThread()) throw new IllegalStateException("Future ran on main");
                System.gc(); return input.take();
            }}) { protected void done() {
                final FutureTask<String> completed = this;
                String value;
                try { value = "Future result: " + get(); }
                catch (CancellationException expected) { value = "Future worker cancelled"; }
                catch (Exception failure) { value = "Future worker failed"; }
                final String message = value;
                owner.shutdown();
                timerHandler.post(new Runnable() { public void run() {
                    if (Thread.currentThread() != Looper.getMainLooper().getThread()) throw new IllegalStateException("Future result off main");
                    if (futureTask == completed) label.setText(message);
                }});
            }};
            label.setText("Future waiting for input"); owner.execute(futureTask);
        }}); layout.addView(startFuture);
        Button deliverFuture = new Button(this); deliverFuture.setText("Deliver future input");
        deliverFuture.setOnClickListener(new View.OnClickListener() { public void onClick(View view) {
            if (futureInput != null) try { futureInput.put("payload"); } catch (InterruptedException failure) { throw new IllegalStateException("main interrupted", failure); }
        }}); layout.addView(deliverFuture);
        Button cancelFuture = new Button(this); cancelFuture.setText("Cancel future worker");
        cancelFuture.setOnClickListener(new View.OnClickListener() { public void onClick(View view) {
            if (futureTask != null) futureTask.cancel(true);
            if (futurePool != null) futurePool.shutdownNow();
        }}); layout.addView(cancelFuture);
        setContentView(layout);
    }
    public void onDestroy() {
        if (looperRunning) WorkerLooperContract.quit(0);
        if (backgroundTimer != null) backgroundTimer.cancel();
        FutureTask<String> task = futureTask; futureTask = null;
        if (task != null) task.cancel(true);
        if (futurePool != null) futurePool.shutdownNow();
        timerHandler.removeCallbacksAndMessages(null); super.onDestroy();
    }
    public static void prepare() {
        events = ""; dispatches = 0; messageCallbacks = 0; handled = 0;
        handler = new RecordingHandler();
        final Handler owner = handler;
        Append first = new Append("A"); Append second = new Append("B");
        if (!handler.postDelayed(first,10) || !handler.postAtTime(second,10) || !handler.hasCallbacks(first)) throw new IllegalStateException("post failed");
        handler.post(new Runnable() { public void run() {
            events += "I"; owner.post(new Append("R"));
        }});
        handler.postDelayed(new Append("N"),-100);
        Message empty = handler.obtainMessage(123, null);
        if (empty.what != 123 || empty.obj != null || empty.getTarget() != handler) throw new IllegalStateException("null message payload");
        Message untargeted = Message.obtain(null, -5, null);
        if (untargeted.getTarget() != null || untargeted.what != -5 || untargeted.obj != null
                || untargeted.arg1 != 0 || untargeted.arg2 != 0 || untargeted.getCallback() != null || untargeted.getWhen() != 0)
            throw new IllegalStateException("static message defaults");
        Object payload = new StringBuilder().append("pay").append("load").toString();
        queued = Message.obtain(handler, 7, payload); queued.arg1=11; queued.arg2=12;
        System.gc();
        if (queued.obj != payload) throw new IllegalStateException("message payload identity/GC");
        if (queued.getTarget()!=handler || !handler.sendMessageAtTime(queued,10) || queued.getWhen()!=10) throw new IllegalStateException("message timing");
        boolean rejected = false;
        try { handler.sendMessage(queued); } catch (IllegalStateException expected) { rejected=true; }
        if (!rejected) throw new IllegalStateException("double enqueue accepted");
        handler.sendMessageDelayed(handler.obtainMessage(9, -11, -12, payload),10);
        Append removed = new Append("BAD"); handler.postDelayed(removed,10); handler.removeCallbacks(removed);
        if (handler.hasCallbacks(removed)) throw new IllegalStateException("callback removal");
        Object token = new EqualToken(); Object equalToken = new EqualToken();
        handler.postAtTime(new Append("BAD"),token,10);
        handler.postAtTime(new Append("T"),equalToken,10);
        handler.removeCallbacksAndMessages(token);
        handler = null; queued = null; // the queue must retain Handler, callback, token and Message.
    }
    public static int result() {
        return "INRABMCT".equals(events) && dispatches==8 && messageCallbacks==2 && handled==1 && SystemClock.uptimeMillis()==10 && SystemClock.elapsedRealtime()==10 ? 1 : 0;
    }
    public static String readEvents() { return events; }
    public static void prepareFault() {
        Handler local = new Handler(); local.post(new Runnable() { public void run() { throw new IllegalStateException("timer failure"); } });
        local.postDelayed(new Append("after fault"),1);
    }
    public static void prepareSpin() {
        final Handler local = new Handler(); local.post(new Runnable() { public void run() { local.post(this); } });
    }
    public static void prepareCapacity() {
        Handler local = new Handler(); Runnable task = new Append("bounded");
        for (int i=0; i<16385; i++) local.postDelayed(task,100000);
    }
    public static void prepareLastGc() {
        new RecordingHandler().post(new Append("last"));
    }
    public static int postAfterClose() { return new Handler().post(new Append("closed")) ? 0 : 1; }
}
