package com.node.v8;

public class V8Context {

    public static native void init();
    public static native V8Context create();
    public native V8Result eval(String script);
    public native void set(String key, int[] arr);

    private long runtimePtr;

    public V8Context(long runtimePtr) {
        this.runtimePtr = runtimePtr;
    }

    public static class V8Result {
        long resultPtr;
        long runtimePtr;

        public V8Result(long resultPtr, long runtimePtr) {
            this.resultPtr = resultPtr;
            this.runtimePtr = runtimePtr;
        }

        public native Integer[] toIntegerArray();
        public native Integer toInteger();
    }
}
