package com.phonebridge2.app.pairing

import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json

/** Canonical JSON codec for the newline-delimited PhoneBridge protocol. */
object ProtocolJson {
    private val json = Json {
        classDiscriminator = "type"
        ignoreUnknownKeys = false
        encodeDefaults = true
    }

    fun encode(message: Message): String = json.encodeToString(message)

    fun decode(line: String): Message = json.decodeFromString(line.trim())
}
