package com.phonebridge2.app.pairing

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

/**
 * Зеркало pc/src/protocol.rs::Message. Формат сериализации должен совпадать
 * byte-in-byte с Rust-стороной (serde с tag="type", content="data") —
 * при любом изменении полей менять ОБА файла синхронно, иначе PC и телефон
 * перестанут понимать друг друга молча (без явной ошибки).
 */
@Serializable
sealed class Message {

    @Serializable
    @SerialName("Hello")
    data class Hello(
        val data: HelloData
    ) : Message()

    @Serializable
    data class HelloData(
        val device_id: String,
        val device_name: String,
        val platform: String = "android",
        val protocol_version: Int = 1
    )

    @Serializable
    @SerialName("HelloAck")
    data class HelloAck(val data: HelloAckData) : Message()

    @Serializable
    data class HelloAckData(
        val device_id: String,
        val device_name: String,
        val trusted: Boolean
    )

    @Serializable
    @SerialName("Ping")
    object Ping : Message()

    @Serializable
    @SerialName("Pong")
    object Pong : Message()

    @Serializable
    @SerialName("IncomingCall")
    data class IncomingCall(val data: IncomingCallData) : Message()

    @Serializable
    data class IncomingCallData(
        val caller_number: String? = null,
        val caller_name: String? = null
    )

    @Serializable
    @SerialName("CallEnded")
    object CallEnded : Message()

    @Serializable
    @SerialName("CallAnswer")
    object CallAnswer : Message()

    @Serializable
    @SerialName("CallDecline")
    object CallDecline : Message()

    @Serializable
    @SerialName("Error")
    data class Error(val data: ErrorData) : Message()

    @Serializable
    data class ErrorData(val message: String)
}

/**
 * NOTE: реальный serde-тег на Rust-стороне сериализует enum-варианты без полей
 * (Ping/Pong/CallEnded/...) как {"type":"Ping"} БЕЗ поля "data". kotlinx.serialization
 * с polymorphic sealed class по умолчанию сериализует object как {"type":"Ping"} —
 * это должно совпасть, но перед первым реальным подключением стоит явно сверить
 * JSON на обеих сторонах юнит-тестом (TODO для Kimi), а не полагаться на "похоже".
 */
