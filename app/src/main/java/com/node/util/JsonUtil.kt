package com.node.util

import kotlinx.serialization.Serializable
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.json.Json

@Serializable
data class Version(
    val http_parser: String,
    val mobile: String,
    val node: String,
    val v8: String,
    val uv: String,
    val zlib: String,
    val ares: String,
    val modules: String,
    val nghttp2: String,
    val openssl: String
)

class JsonUtil {
    companion object {

        @JvmStatic
        fun parseVersion(raw: String): Version {
            return Json.decodeFromString(raw)
        }
    }
}
