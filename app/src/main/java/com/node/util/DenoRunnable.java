package com.node.util;

import androidx.annotation.Keep;

@Keep
public class DenoRunnable implements Runnable {

    private long task;

    public DenoRunnable(long task) {
        this.task = task;
    }

    protected native void invoke(long task);

    @Override
    public void run() {
        invoke(task);
    }
}
