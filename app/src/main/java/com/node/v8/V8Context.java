package com.node.v8;

public class V8Context {

    public static native void init();
    public static native String eval(String script);
    public static native void set(String key, int[] arr);

}
