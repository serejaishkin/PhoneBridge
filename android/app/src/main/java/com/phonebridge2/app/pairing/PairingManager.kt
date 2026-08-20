package com.phonebridge2.app.pairing

import android.content.Context
import com.phonebridge2.app.connection.ConnectionManager
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow

/**
 * Owns application-level pairing state. TLS pins the PC identity;
 * this layer handles the human confirmation step and persists two-way trust.
 */
class PairingManager(
    context: Context,
    private val identity: Identity,
    private val connection: ConnectionManager,
) {
    sealed class State {
        data object Idle : State()
        data class WaitingForConfirmation(val pcDeviceId: String, val pcFingerprint: String, val shortCode: String) : State()
        data class Paired(val pcDeviceId: String, val pcFingerprint: String) : State()
        data class Failed(val message: String) : State()
    }

    private val trustStore = TrustStore(context)
    private val _state = MutableStateFlow<State>(State.Idle)
    val state: StateFlow<State> = _state.asStateFlow()

    private var pendingPcDeviceId: String? = null
    private var pendingPcFingerprint: String? = null
    private var pendingShortCode: String? = null

    fun hello(deviceName: String): Message.Hello = Message.Hello(
        data = Message.HelloData(device_id = identity.deviceId, device_name = deviceName, platform = "android", protocol_version = 1, fingerprint = identity.fingerprintHex())
    )

    fun onMessage(message: Message) {
        when (message) {
            is Message.HelloAck -> {
                if (message.data.protocol_version != 1) { _state.value = State.Failed("Unsupported PC protocol version"); return }
                if (message.data.trusted) trustPc(message.data.device_id, message.data.fingerprint)
            }
            is Message.PairChallenge -> {
                pendingPcDeviceId = message.data.device_id
                pendingPcFingerprint = message.data.fingerprint
                pendingShortCode = message.data.short_code
                _state.value = State.WaitingForConfirmation(message.data.device_id, message.data.fingerprint, message.data.short_code)
            }
            is Message.PairApprove -> {
                val pendingId = pendingPcDeviceId
                val pendingCode = pendingShortCode
                if (pendingId != message.data.device_id || pendingCode != message.data.short_code) {
                    _state.value = State.Failed("PC pairing approval does not match the pending challenge")
                    return
                }
                val fingerprint = pendingPcFingerprint
                if (fingerprint == null) _state.value = State.Failed("PC approved pairing without a pending fingerprint")
                else {
                    trustPc(message.data.device_id, fingerprint)
                    connection.send(Message.PairResult(Message.PairResultData(identity.deviceId, true, "Android accepted PC pairing approval")))
                }
            }
            is Message.PairReject -> {
                if (pendingPcDeviceId == message.data.device_id) {
                    clearPending()
                    _state.value = State.Failed(message.data.reason)
                }
            }
            is Message.PairResult -> {
                if (message.data.trusted) {
                    val fingerprint = pendingPcFingerprint
                    if (fingerprint == null) _state.value = State.Failed("Pairing succeeded without a pending PC fingerprint")
                    else trustPc(message.data.device_id, fingerprint)
                } else _state.value = State.Failed(message.data.message)
            }
            else -> Unit
        }
    }

    /** Call only after the user has visually compared the code on both devices. */
    fun confirmPairing() {
        val deviceId = pendingPcDeviceId
        val code = pendingShortCode
        if (deviceId == null || code == null) { _state.value = State.Failed("No pending pairing challenge"); return }
        connection.send(Message.PairConfirm(Message.PairConfirmData(device_id = identity.deviceId, short_code = code)))
    }

    fun requestPairing() {
        connection.send(Message.PairRequest(Message.PairRequestData(device_id = identity.deviceId, device_name = identity.deviceId, fingerprint = identity.fingerprintHex())))
    }

    fun isTrustedPc(deviceId: String, fingerprint: String): Boolean = trustStore.isTrusted(deviceId, fingerprint)

    /** Explicitly removes local trust for a PC. The next connection requires human confirmation again. */
    fun forgetPc(deviceId: String) {
        trustStore.revoke(deviceId)
        if ((state.value as? State.Paired)?.pcDeviceId == deviceId) {
            clearPending()
            _state.value = State.Idle
        }
    }

    private fun trustPc(deviceId: String, fingerprint: String) {
        trustStore.trust(deviceId, fingerprint)
        clearPending()
        _state.value = State.Paired(deviceId, fingerprint)
    }

    private fun clearPending() { pendingPcDeviceId = null; pendingPcFingerprint = null; pendingShortCode = null }
}
