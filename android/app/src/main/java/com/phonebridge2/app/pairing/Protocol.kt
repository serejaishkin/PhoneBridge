package com.phonebridge2.app.pairing

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

/** Зеркало pc/src/protocol.rs::Message. */
@Serializable
sealed class Message {
    @Serializable
    @SerialName("Hello")
    data class Hello(val data: HelloData) : Message()

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
    @SerialName("MediaCommand")
    data class MediaCommand(val data: MediaCommandData) : Message()

    @Serializable
    data class MediaCommandData(val command: MediaCommandType)

    @Serializable
    enum class MediaCommandType {
        Play,
        Pause,
        PlayPause,
        Next,
        Previous
    }

    @Serializable
    @SerialName("MediaState")
    data class MediaState(val data: MediaStateData) : Message()

    @Serializable
    data class MediaStateData(
        @SerialName("package") val packageName: String? = null,
        val state: MediaPlaybackState = MediaPlaybackState.Unknown,
        val title: String? = null,
        val artist: String? = null,
        val album: String? = null
    )

    @Serializable
    enum class MediaPlaybackState {
        Playing,
        Paused,
        Buffering,
        None,
        Unknown
    }

    @Serializable
    @SerialName("PhoneBluetoothStatus")
    data class PhoneBluetoothStatus(val data: PhoneBluetoothStatusData) : Message()

    @Serializable
    data class PhoneBluetoothStatusData(val hfp_calls_toggle_enabled: Boolean)

    @Serializable
    @SerialName("PcBluetoothStatus")
    data class PcBluetoothStatus(val data: PcBluetoothStatusData) : Message()

    @Serializable
    data class PcBluetoothStatusData(val hfp_supported: HfpSupport)

    @Serializable
    enum class HfpSupport {
        Supported,
        Unsupported,
        Unknown
    }

    @Serializable
    @SerialName("Error")
    data class Error(val data: ErrorData) : Message()

    @Serializable
    data class ErrorData(val message: String)
}
