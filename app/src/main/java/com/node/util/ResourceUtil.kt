package com.node.util

import android.content.Context
import androidx.annotation.Keep
import java.lang.ref.WeakReference

@Keep
class ResourceUtil {

    companion object {
        private var context_: WeakReference<Context>? = null

        @JvmStatic
        fun setContext(context: Context) {
            this.context_ = WeakReference(context)
        }

        private fun getResourceId(context: Context, name: String): Int {
            return context.resources.getIdentifier(name, "raw", context.packageName)
        }
    }

}