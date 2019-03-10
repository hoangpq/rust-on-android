package com.node.v8;

public class V8Runnable implements Runnable {

    private V8Context ctx_;
    private long fn_;
    private long time_;
    private TimerType type_;

    private V8Runnable(V8Context ctx, long fn, long time, TimerType type) {
        this.ctx_ = ctx;
        this.fn_ = fn;
        this.time_ = time;
        this.type_ = type;
    }

    public static V8Runnable createTimeoutRunnable(V8Context ctx, long fn, long time) {
        return new V8Runnable(ctx, fn, time, TimerType.TIMEOUT);
    }

    public static V8Runnable createIntervalRunnable(V8Context ctx, long fn, long time) {
        return new V8Runnable(ctx, fn, time, TimerType.INTERVAL);
    }

    @Override
    public void run() {
        this.ctx_.callFn(this.fn_, TimerType.INTERVAL == this.type_, time_);
    }
}
