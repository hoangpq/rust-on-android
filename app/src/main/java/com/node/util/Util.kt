package com.node.util

import android.graphics.Bitmap
import android.support.annotation.Keep

class Util {

    companion object {

        // Must be run on main thread
        external fun createReference(refName: String)

        @JvmStatic
        @Keep
        fun createBitmap(width: Int, height: Int): Bitmap = Bitmap.createBitmap(width, height, Bitmap.Config.ARGB_8888)
    }
}
