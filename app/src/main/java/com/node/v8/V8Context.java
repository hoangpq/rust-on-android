package com.node.v8;

import android.util.SparseArray;

public class V8Context {

    public static native V8Context create();
    public native V8Result eval(String script);
    public native void set(String key, int[] arr);
    public native void callFn(long fn, boolean interval, long time);

    private static SparseArray<V8Context> hash_;
    private static int current_index = 0;
    private long runtime__;

    static {
        hash_ = new SparseArray<>();
    }

    public V8Context(long runtime__) {
        this.runtime__ = runtime__;
        hash_.put(++current_index, this);
    }

    public static V8Context getCurrent() {
        return hash_.get(current_index);
    }

    public static class V8Result {
        long result__;
        long runtime__;

        public V8Result(long result__, long runtime__) {
            this.result__ = result__;
            this.runtime__ = runtime__;
        }

        public native Integer[] toIntegerArray();
        public native Integer toInteger();
        public native String toJavaString();

        @Override
        public String toString() {
            return toJavaString();
        }
    }
}
