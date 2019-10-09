package com.node.util

import android.graphics.Bitmap
import android.support.annotation.Keep
import android.util.Log

class Util {

    companion object {

        // Must be run on main thread
        external fun createReference(refName: String)

        @Keep
        @JvmStatic
        fun createBitmap(width: Int, height: Int): Bitmap = Bitmap.createBitmap(width, height, Bitmap.Config.ARGB_8888)

        @Keep
        @JvmStatic
        fun testMethod(x: Int): Int {
            Log.d("test fn", (x + 10).toString())
            return x + 10
        }
    }
}
