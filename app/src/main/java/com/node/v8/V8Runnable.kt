package com.node.v8

class V8Runnable private constructor(private val ctx_: V8Context, private val fn_: Long, private val time_: Long, private val type_: TimerType) : Runnable {

    override fun run() {
        this.ctx_.callFn(this.fn_, TimerType.INTERVAL == this.type_, time_)
    }

    companion object {

        fun createTimeoutRunnable(ctx: V8Context, fn: Long, time: Long): V8Runnable {
            return V8Runnable(ctx, fn, time, TimerType.TIMEOUT)
        }

        fun createIntervalRunnable(ctx: V8Context, fn: Long, time: Long): V8Runnable {
            return V8Runnable(ctx, fn, time, TimerType.INTERVAL)
        }
    }
}
