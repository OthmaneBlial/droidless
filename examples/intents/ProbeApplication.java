package org.droidless.intents;

import android.app.Activity;
import android.app.Application;
import android.os.Bundle;

/** Compiled observer checks; never independent app compatibility evidence. */
public class ProbeApplication extends Application {
    static ProbeApplication instance;
    static String events="";
    public void onCreate() {
        super.onCreate(); instance=this;
        registerActivityLifecycleCallbacks(null); unregisterActivityLifecycleCallbacks(null);
        Observer victim=new Observer(new StringBuilder().append("removed").toString());
        registerActivityLifecycleCallbacks(new Mutator(victim));
        registerActivityLifecycleCallbacks(victim);
        registerActivityLifecycleCallbacks(new Observer("permanent"));
    }
    static class Observer implements ActivityLifecycleCallbacks {
        final String name;
        Observer(String name) { this.name=name; }
        void record(Activity activity, String event) {
            if (activity.getApplication()!=instance || activity.getApplicationContext()!=instance)
                throw new IllegalStateException("Application identity changed");
            String screen=activity instanceof MainActivity ? "home" : activity instanceof MainActivity.Detail ? "detail" : "guarded";
            events+=name+":"+screen+":"+event+";";
            System.gc();
        }
        public void onActivityCreated(Activity a, Bundle b) {
            if (b!=null || (a instanceof MainActivity && !MainActivity.creating))
                throw new IllegalStateException("Creation observer called outside super");
            record(a,"create");
        }
        public void onActivityStarted(Activity a) {
            record(a,"start");
            if ("fault".equals(name)) {
                instance.unregisterActivityLifecycleCallbacks(this); System.gc();
                throw new IllegalStateException("observer failed");
            }
        }
        public void onActivityResumed(Activity a) { record(a,"resume"); }
        public void onActivityPaused(Activity a) { record(a,"pause"); }
        public void onActivityStopped(Activity a) { record(a,"stop"); }
        public void onActivityDestroyed(Activity a) { record(a,"destroy"); }
        public void onActivitySaveInstanceState(Activity a, Bundle b) {
            throw new IllegalStateException("Unexpected saved-state dispatch");
        }
    }
    static class Mutator extends Observer {
        Observer victim;
        Mutator(Observer victim) { super("mutator"); this.victim=victim; }
        public void onActivityCreated(Activity a, Bundle b) {
            super.onActivityCreated(a,b);
            instance.unregisterActivityLifecycleCallbacks(this);
            instance.unregisterActivityLifecycleCallbacks(victim); victim=null;
            instance.registerActivityLifecycleCallbacks(new Observer("late")); System.gc();
        }
    }
    public static String eventLog() { return events; }
    public static Object registerTransient() {
        Observer observer=new Observer("transient"); instance.registerActivityLifecycleCallbacks(observer); return observer;
    }
    public static Object registerFault() {
        Observer observer=new Observer("fault"); instance.registerActivityLifecycleCallbacks(observer); return observer;
    }
    public static Application application() { return instance; }
}
