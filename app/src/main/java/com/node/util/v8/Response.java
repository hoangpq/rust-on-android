package com.node.util.v8;

import android.support.annotation.Keep;
import android.support.annotation.Nullable;

@Keep
public class Response {
    private Object internal;
    private int sig;

    public Response(@Nullable Object internal, int sig) {
        this.internal = internal;
        this.sig = sig;
    }

    public static Response newNull() {
        return new Response(null, -1);
    }

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
