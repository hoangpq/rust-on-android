package com.node.v8;

public class V8Context {

    public static native V8Context create();
    public native V8Result eval(String script);
    public native void set(String key, int[] arr);
    public native void callFn(long fn);

    public V8Context(long runtime__) {
        this.runtime__ = runtime__;
    }
    private long runtime__;

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
