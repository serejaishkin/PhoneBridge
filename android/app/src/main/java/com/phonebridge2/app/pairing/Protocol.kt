package com.phonebridge2.app.pairing

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

/** Exact wire mirror of pc/src/protocol.rs. */
@Serializable
sealed class Message {
    @Serializable @SerialName("Hello") data class Hello(val data: HelloData) : Message()
    @Serializable data class HelloData(val device_id: String, val device_name: String, val platform: String = "android", val protocol_version: Int = 1, val fingerprint: String)
    @Serializable @SerialName("HelloAck") data class HelloAck(val data: HelloAckData) : Message()
    @Serializable data class HelloAckData(val device_id: String, val device_name: String, val protocol_version: Int, val trusted: Boolean, val fingerprint: String)
    @Serializable @SerialName("PairRequest") data class PairRequest(val data: PairRequestData) : Message()
    @Serializable data class PairRequestData(val device_id: String, val device_name: String, val fingerprint: String)
    @Serializable @SerialName("PairChallenge") data class PairChallenge(val data: PairChallengeData) : Message()
    @Serializable data class PairChallengeData(val device_id: String, val fingerprint: String, val short_code: String) : MessageData
    @Serializable @SerialName("PairConfirm") data class PairConfirm(val data: PairConfirmData) : Message()
    @Serializable data class PairConfirmData(val device_id: String, val short_code: String) : MessageData
    @Serializable @SerialName("PairResult") data class PairResult(val data: PairResultData) : Message()
    @Serializable data class PairResultData(val device_id: String, val trusted: Boolean, val message: String) : MessageData
    @Serializable @SerialName("Ping") object Ping : Message()
    @Serializable @SerialName("Pong") object Pong : Message()
    @Serializable @SerialName("Disconnect") data class Disconnect(val data: DisconnectData) : Message()
    @Serializable data class DisconnectData(val reason: String) : MessageData
    @Serializable @SerialName("IncomingCall") data class IncomingCall(val data: IncomingCallData) : Message()
    @Serializable data class IncomingCallData(val caller_number: String? = null, val caller_name: String? = null) : MessageData
    @Serializable @SerialName("CallEnded") object CallEnded : Message()
    @Serializable @SerialName("CallAnswer") object CallAnswer : Message()
    @Serializable @SerialName("CallDecline") object CallDecline : Message()
    @Serializable @SerialName("PhoneBluetoothStatus") data class PhoneBluetoothStatus(val data: PhoneBluetoothStatusData) : Message()
    @Serializable data class PhoneBluetoothStatusData(val hfp_calls_toggle_enabled: Boolean) : MessageData
    @Serializable @SerialName("PcBluetoothStatus") data class PcBluetoothStatus(val data: PcBluetoothStatusData) : Message()
    @Serializable data class PcBluetoothStatusData(val hfp_supported: HfpSupport) : MessageData
    @Serializable @SerialName("Error") data class Error(val data: ErrorData) : Message()
    @Serializable data class ErrorData(val message: String) : MessageData
}

interface MessageData

@Serializable
enum class HfpSupport { Supported, Unsupported, Unknown }
