package com.phonebridge.app.call

import android.content.Context
import android.telephony.PhoneStateListener
import android.telephony.TelephonyManager
import com.phonebridge.app.network.SignalingClient
import kotlinx.coroutines.*

class CallManager(context: Context) {

    private val telephonyManager = context.getSystemService(Context.TELEPHONY_SERVICE) as TelephonyManager
    private val signalingClient = SignalingClient()
    private val scope = CoroutineScope(Dispatchers.IO + SupervisorJob())

    private val phoneStateListener = object : PhoneStateListener() {
        override fun onCallStateChanged(state: Int, phoneNumber: String?) {
            when (state) {
                TelephonyManager.CALL_STATE_RINGING -> {
                    scope.launch {
                        signalingClient.sendEvent("incoming_call", mapOf(
                            "number" to (phoneNumber ?: "Unknown")
                        ))
                    }
                }
                TelephonyManager.CALL_STATE_OFFHOOK -> {
                    scope.launch {
                        signalingClient.sendEvent("call_answered", emptyMap())
                    }
                }
                TelephonyManager.CALL_STATE_IDLE -> {
                    scope.launch {
                        signalingClient.sendEvent("call_ended", emptyMap())
                    }
                }
            }
        }
    }

    fun start() {
        telephonyManager.listen(phoneStateListener, PhoneStateListener.LISTEN_CALL_STATE)
        scope.launch {
            signalingClient.connect("ws://192.168.137.1:5000")
        }
    }

    fun stop() {
        telephonyManager.listen(phoneStateListener, PhoneStateListener.LISTEN_NONE)
        scope.cancel()
    }

    fun answerCall() {
        // Requires Default Dialer or accessibility service on non-rooted devices
    }

    fun endCall() {
        // Requires Default Dialer or accessibility service on non-rooted devices
    }
}
