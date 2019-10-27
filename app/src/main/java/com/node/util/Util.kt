package com.node.util

import android.graphics.Bitmap
import android.support.annotation.Keep

class Util {

    companion object {

        @JvmStatic
        external fun createReference(refName: String)

        @Keep
        @JvmStatic
        fun createBitmap(width: Int, height: Int): Bitmap = Bitmap.createBitmap(width, height, Bitmap.Config.ARGB_8888)
    }
}
