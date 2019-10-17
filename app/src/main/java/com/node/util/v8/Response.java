package com.node.util.v8;

import android.support.annotation.Keep;
import android.support.annotation.Nullable;

import com.node.util.JNIHelper;

@Keep
public class Response {
    private Object internal;
    private int sig;
    private boolean hasError;

    public Response(@Nullable Object internal, int sig) {
        this.internal = internal;
        this.sig = sig;
        this.hasError = false;
    }

    public Response(@Nullable Object internal, int sig, boolean hasError) {
        this.internal = internal;
        this.sig = sig;
        this.hasError = hasError;
    }

    public static Response newError(String message) {
        return new Response(message, JNIHelper.getIndexByClass(String.class), true);
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
