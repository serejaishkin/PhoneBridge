package com.phonebridge.app.call

import android.Manifest
import android.content.Context
import android.content.pm.PackageManager
import android.telecom.TelecomManager
import android.telephony.PhoneStateListener
import android.telephony.TelephonyManager
import androidx.core.content.ContextCompat
import com.phonebridge.app.media.MediaControllerBridge
import com.phonebridge.app.network.SignalingClient
import com.phonebridge.app.sms.SmsBridge

/** Phone-side control endpoint for calls + media + SMS. */
class CallManager(private val context: Context) {
    private val telephonyManager = context.getSystemService(Context.TELEPHONY_SERVICE) as TelephonyManager
    private val signalingClient = SignalingClient { type, data ->
        when (type) {
            "call_answer" -> answerCall()
            "call_decline" -> endCall()
            "media_command" -> MediaControllerBridge.handleCommand(data["command"]?.let { "media_${it.lowercase()}" } ?: "")
            "sms_send" -> SmsBridge.sendFromCommand(data)
            "sms_list" -> SmsBridge.publishRecent()
        }
    }

    private val phoneStateListener = object : PhoneStateListener() {
        override fun onCallStateChanged(state: Int, phoneNumber: String?) {
            when (state) {
                TelephonyManager.CALL_STATE_RINGING -> signalingClient.sendEvent(
                    "incoming_call", mapOf("number" to (phoneNumber ?: "Unknown"))
                )
                TelephonyManager.CALL_STATE_IDLE -> signalingClient.sendEvent("call_ended", emptyMap())
            }
        }
    }

    fun start() {
        telephonyManager.listen(phoneStateListener, PhoneStateListener.LISTEN_CALL_STATE)
        signalingClient.connect(SignalingClient.DEFAULT_URL)
        // Give the PC an initial media snapshot as soon as the control channel opens.
        MediaControllerBridge.refresh()
    }

    fun stop() {
        telephonyManager.listen(phoneStateListener, PhoneStateListener.LISTEN_NONE)
        signalingClient.disconnect()
    }

    fun answerCall() {
        if (ContextCompat.checkSelfPermission(context, Manifest.permission.ANSWER_PHONE_CALLS) != PackageManager.PERMISSION_GRANTED) return
        val telecom = context.getSystemService(Context.TELECOM_SERVICE) as TelecomManager
        try {
            telecom.acceptRingingCall()
        } catch (_: SecurityException) {
            // OEM/default-dialer restrictions can still reject this operation.
        }
    }

    fun endCall() {
        if (ContextCompat.checkSelfPermission(context, Manifest.permission.ANSWER_PHONE_CALLS) != PackageManager.PERMISSION_GRANTED) return
        val telecom = context.getSystemService(Context.TELECOM_SERVICE) as TelecomManager
        try {
            telecom.endCall()
        } catch (_: SecurityException) {
            // OEM/default-dialer restrictions can still reject this operation.
        }
    }
}
