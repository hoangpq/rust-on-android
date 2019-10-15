package com.node.util.v8;

import android.support.annotation.Keep;

@Keep
public class Response {
    private Object internal;
    private int sig;

    public Object getInternal() {
        return internal;
    }

    public void setInternal(Object internal) {
        this.internal = internal;
    }

    public int getSig() {
        return sig;
    }

    public void setSig(int sig) {
        this.sig = sig;
    }
}
