package com.node.v8;

import android.util.Log;

public class V8Runnable implements Runnable {

    private V8Context ctx_;
    private long fn_;

    public V8Runnable(V8Context ctx, long fn) {
        this.ctx_ = ctx;
        this.fn_ = fn;
    }

    @Override
    public void run() {
        Log.d("V8 Runtime", "Invoke fn " + this.fn_);
        this.ctx_.callFn(this.fn_);
    }
}
