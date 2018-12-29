package com.node.v8;

public class V8Context {

    public static native void init();

    public static native V8Context create();

    public native String eval(String script);

    public native void set(String key, int[] arr);

    private long runtimePtr;

    public V8Context(long ptr) {
        this.runtimePtr = ptr;
    }
}
