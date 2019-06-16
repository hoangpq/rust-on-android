package com.node.util

import kotlinx.serialization.*
import kotlinx.serialization.json.*

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

        val JSON: Json by lazy { Json(JsonConfiguration.Stable) }

        @JvmStatic
        fun parseVersion(raw: String): Version {
            return JSON.parse(Version.serializer(), raw)
        }
    }
}
