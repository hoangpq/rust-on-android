package com.node.util

import java.net.URL

class RestUtil {

    companion object {
        @JvmStatic
        fun fetch(url: String): String = URL(url).readText()
    }
}
