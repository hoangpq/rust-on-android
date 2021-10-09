package com.node.sample;

import androidx.annotation.Keep;

import com.node.annotations.CalledByNative;

@Keep
public class Observable {
    @CalledByNative
    public void subscribe() {
    }

    @CalledByNative
    public void subscribe(Object arg) {
    }

    @CalledByNative
    public void subscribe(int arg) {
    }

    @CalledByNative
    public void subscribe(String arg) {
    }
}
