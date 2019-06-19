package com.node.util

import android.graphics.Bitmap

class Util {

    companion object {

        // Must be run on main thread
        external fun createReference(refName: String)

        external fun createWeakRef(refName: String, instance: Any)

        @JvmStatic
        fun createBitmap(width: Int, height: Int): Bitmap = Bitmap.createBitmap(width, height, Bitmap.Config.ARGB_8888)
    }
}
