package com.node.util

import android.content.Context
import android.os.Build
import android.support.annotation.RequiresApi
import com.node.v8.V8Context
import java.io.ByteArrayOutputStream
import java.io.IOException
import java.util.*

object ScriptUtils {

    private fun readFileFromRawDirectory(context: Context, resourceId: Int): String {
        val iStream = context.resources.openRawResource(resourceId)
        var byteStream: ByteArrayOutputStream? = null
        try {
            val buffer = ByteArray(iStream.available())
            iStream.read(buffer)
            byteStream = ByteArrayOutputStream()
            byteStream.write(buffer)
            byteStream.close()
            iStream.close()
        } catch (e: IOException) {
            e.printStackTrace()
        }

        assert(byteStream != null)
        return byteStream!!.toString()
    }

    fun require(ctx_: Context, v8ctx_: V8Context, resourceId: Int) {
        v8ctx_.eval(readFileFromRawDirectory(ctx_, resourceId))
    }

    @RequiresApi(api = Build.VERSION_CODES.N)
    fun bulkEval(v8ctx_: V8Context, vararg scripts: String) {
        val script = Arrays.stream(scripts).reduce { sc, c -> sc + c + "\n" }
        script.ifPresent { v8ctx_.eval(it) }
    }
}
