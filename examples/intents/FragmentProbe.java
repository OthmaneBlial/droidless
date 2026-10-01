package org.droidless.intents;

import android.app.Activity;
import android.app.Fragment;
import android.app.FragmentManager;
import android.app.FragmentTransaction;
import android.os.Bundle;
import android.view.LayoutInflater;
import android.view.View;
import android.view.ViewGroup;

/** Authored headless fragment contract; independent APK progress is tested separately. */
public class FragmentProbe extends Fragment {
    private static String events = "";
    private static Activity host;
    private static int reentrant;
    private String label() { return getArguments().getString("label"); }
    private void log(String event) { events += label() + ":" + event + ";"; System.gc(); }
    private static FragmentProbe probe(String label) {
        FragmentProbe fragment = new FragmentProbe();
        Bundle args = new Bundle(); args.putString("label", label); fragment.setArguments(args);
        return fragment;
    }
    public static void install(Activity activity) {
        host = activity;
        FragmentManager manager = activity.getFragmentManager();
        if (manager != activity.getFragmentManager()) throw new AssertionError("manager identity");
        FragmentProbe fragment = probe("initial");
        FragmentTransaction transaction = manager.beginTransaction().add(fragment, "initial");
        if (fragment.getActivity() != null || fragment.isAdded()) throw new AssertionError("premature attachment");
        if (transaction.commit() != -1 || manager.findFragmentByTag("initial") != null)
            throw new AssertionError("commit must queue");
        System.gc();
        if (!manager.executePendingTransactions() || manager.executePendingTransactions())
            throw new AssertionError("pending execution result");
        if (manager.findFragmentByTag(new StringBuilder().append("initial").toString()) != fragment || !fragment.isAdded())
            throw new AssertionError("tag or attachment");
    }
    public static Fragment initial() { return host.getFragmentManager().findFragmentByTag("initial"); }
    public static String eventLog() { return events; }
    public static void addLater() { host.getFragmentManager().beginTransaction().add(probe("late"), "late").commit(); }
    public static int errors() {
        FragmentManager manager = host.getFragmentManager();
        FragmentProbe fragment = probe("errors");
        FragmentTransaction transaction = manager.beginTransaction().add(fragment, "errors");
        transaction.commit();
        int checked = 0;
        try { transaction.commit(); } catch (IllegalStateException expected) { checked++; }
        try { manager.beginTransaction().add(fragment, "changed"); } catch (IllegalStateException expected) { checked++; }
        manager.executePendingTransactions();
        try { fragment.setArguments(new Bundle()); } catch (IllegalStateException expected) { checked++; }
        manager.beginTransaction().add(fragment, "errors").commit();
        try { manager.executePendingTransactions(); } catch (IllegalStateException expected) { checked++; }
        if (manager.executePendingTransactions()) throw new AssertionError("failed dispatch retained queue");
        Recursive recursive = new Recursive();
        Bundle args = new Bundle(); args.putString("label", "recursive"); recursive.setArguments(args);
        manager.beginTransaction().add(recursive, "recursive").commit(); manager.executePendingTransactions();
        Failure failure = new Failure(); failure.setArguments(args);
        manager.beginTransaction().add(failure, "failure").commit();
        try { manager.executePendingTransactions(); } catch (IllegalStateException expected) { checked++; }
        if (manager.executePendingTransactions()) throw new AssertionError("callback failure retained queue");
        return checked + reentrant;
    }
    public static class Recursive extends FragmentProbe {
        public void onCreate(Bundle state) {
            super.onCreate(state);
            try { getFragmentManager().executePendingTransactions(); }
            catch (IllegalStateException expected) { reentrant++; }
        }
    }
    public static class Failure extends FragmentProbe {
        public void onCreate(Bundle state) { super.onCreate(state); throw new IllegalStateException("fragment callback failed"); }
    }
    public void onAttach(Activity activity) {
        super.onAttach(activity);
        if (getActivity() != activity || !isAdded()) throw new AssertionError("onAttach ownership");
        log("attach");
    }
    public void onCreate(Bundle state) { super.onCreate(state); log("create"); }
    public View onCreateView(LayoutInflater inflater, ViewGroup container, Bundle state) {
        if (inflater == null || container != null || state != null) throw new AssertionError("headless createView");
        log("view"); return null;
    }
    public void onActivityCreated(Bundle state) {
        super.onActivityCreated(state);
        if (MainActivity.creating || getActivity().findViewById(R.id.status) == null)
            throw new AssertionError("fragment preceded activity creation");
        log("activity");
    }
    public void onStart() { super.onStart(); log("start"); }
    public void onResume() {
        super.onResume(); if (!isResumed()) throw new AssertionError("resume state"); log("resume");
    }
    public void onPause() {
        super.onPause(); if (isResumed()) throw new AssertionError("pause state"); log("pause");
    }
    public void onStop() { super.onStop(); log("stop"); }
    public void onDestroyView() { super.onDestroyView(); log("destroyView"); }
    public void onDestroy() { super.onDestroy(); log("destroy"); }
    public void onDetach() { super.onDetach(); log("detach"); }
}
