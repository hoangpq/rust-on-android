package com.node.util

import android.content.Context
import android.support.annotation.Keep
import android.util.Log
import com.node.sample.R
import java.io.BufferedReader
import java.io.InputStreamReader
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

        @JvmStatic
        fun readRawResource(name: String): String? {
            Log.d("Kotlin", name)

            val context = context_!!.get()!!

            val resourceId = getResourceId(context, name)
            Log.d("Kotlin", String.format("%d %d", resourceId, R.raw.isolate))

            val stream = context.resources.openRawResource(resourceId)
            val builder = StringBuilder()

            var line: String?
            val br = BufferedReader(InputStreamReader(stream))
            line = br.readLine()
            while (line != null) {
                builder.append(line)
                line = br.readLine()
            }

            br.close()

            return builder.toString()
        }
    }

}