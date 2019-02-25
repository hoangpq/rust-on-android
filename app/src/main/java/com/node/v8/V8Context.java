package com.node.v8;

public class V8Context {

    public static native void init();

    public static native V8Context create();

    public native V8Result eval(String script);

    public native void set(String key, int[] arr);

    private long runtime__;

    public V8Context(long runtime__) {
        this.runtime__ = runtime__;
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
    }
}
