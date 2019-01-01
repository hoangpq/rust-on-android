package com.node.v8;

import com.node.sample.Observable;
import com.node.v8.V8Context.V8Result;

public class V8Promise extends V8Result {

    public V8Promise(long resultPtr, long runtimePtr) {
        super(resultPtr, runtimePtr);
    }

    public native void then(Observable observer);
}
