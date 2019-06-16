package com.node.v8

import android.util.SparseArray

class V8Context(private val runtime__: Long) {
    external fun eval(script: String): V8Result
    external fun setKey(key: String, arr: IntArray)
    external fun callFn(fn: Long, interval: Boolean, time: Long)

    init {
        hash_?.put(++current_index, this)
    }

    class V8Result(internal var result__: Long, internal var runtime__: Long) {
        private external fun toNativeString(): String
        override fun toString(): String = toNativeString()
    }

    var parent: UIUpdater? = null

    companion object {

        @JvmStatic
        external fun create(): V8Context

        private var hash_: SparseArray<V8Context>? = null
        private var current_index = 0

        init {
            hash_ = SparseArray()
        }

        @JvmStatic
        val current: V8Context?
            get() = hash_?.get(current_index)
    }
}
