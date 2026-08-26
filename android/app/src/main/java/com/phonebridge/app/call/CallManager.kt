package com.phonebridge.app.call

import android.Manifest
import android.content.Context
import android.content.pm.PackageManager
import android.os.Handler
import android.os.Looper
import android.telecom.TelecomManager
import android.telephony.PhoneStateListener
import android.telephony.TelephonyManager
import android.widget.Toast
import androidx.core.content.ContextCompat
import com.phonebridge.app.media.MediaControllerBridge
import com.phonebridge.app.network.SignalingClient
import com.phonebridge.app.pairing.PhoneIdentity
import com.phonebridge.app.pairing.TrustStore
import com.phonebridge.app.sms.SmsBridge

/** Phone-side control endpoint for calls + media + SMS. */
class CallManager(private val context: Context) {
    private val mainHandler = Handler(Looper.getMainLooper())

    // Lazy providers evaluated inside the connection worker thread: RSA keygen
    // for the identity must not run on the Android main thread.
    private val identityProvider: () -> PhoneIdentity = { PhoneIdentity.loadOrCreate(context) }
    private val trustProvider: () -> TrustStore = { TrustStore.load(context) }

    private fun toast(message: String) {
        mainHandler.post { Toast.makeText(context, message, Toast.LENGTH_LONG).show() }
    }

    private val signalingClient = SignalingClient(
        onCommand = { type, data ->
            when (type) {
                "call_answer" -> answerCall()
                "call_decline" -> endCall()
                "media_command" -> {
                    val command = when (data["command"]) {
                        "Play" -> "media_play"
                        "Pause" -> "media_pause"
                        "PlayPause" -> "media_play_pause"
                        "Next" -> "media_next"
                        "Previous" -> "media_previous"
                        else -> ""
                    }
                    if (command.isNotEmpty()) MediaControllerBridge.handleCommand(command)
                }
                "sms_send" -> SmsBridge.sendFromCommand(data)
                "sms_list" -> SmsBridge.publishRecent()
            }
        },
        onStatus = { status, _ ->
            when (status) {
                "first_connection" -> toast("Первое подключение к ПК. Сверьте код сопряжения на экране ПК")
                "connected_unverified" -> Unit // already announced via first_connection
                "pairing_rejected" -> toast("ПК отклонил сопряжение")
                "certificate_mismatch" -> toast("Сертификат ПК изменился! Подключение прервано")
                else -> Unit
            }
        },
        identityProvider = identityProvider,
        trustProvider = trustProvider
    )

    private val phoneStateListener = object : PhoneStateListener() {
        override fun onCallStateChanged(state: Int, phoneNumber: String?) {
            when (state) {
                TelephonyManager.CALL_STATE_RINGING -> signalingClient.sendEvent("incoming_call", mapOf("number" to (phoneNumber ?: "Unknown")))
                TelephonyManager.CALL_STATE_IDLE -> signalingClient.sendEvent("call_ended", emptyMap())
            }
        }
    }

    fun start(host: String = "192.168.137.1") {
        telephonyManager.listen(phoneStateListener, PhoneStateListener.LISTEN_CALL_STATE)
        signalingClient.connect("tls://$host:17591")
        MediaControllerBridge.refresh()
    }

    fun stop() {
        telephonyManager.listen(phoneStateListener, PhoneStateListener.LISTEN_NONE)
        signalingClient.disconnect()
    }

    fun answerCall() {
        if (ContextCompat.checkSelfPermission(context, Manifest.permission.ANSWER_PHONE_CALLS) != PackageManager.PERMISSION_GRANTED) return
        val telecom = context.getSystemService(Context.TELECOM_SERVICE) as TelecomManager
        runCatching { telecom.acceptRingingCall() }
    }

    fun endCall() {
        if (ContextCompat.checkSelfPermission(context, Manifest.permission.ANSWER_PHONE_CALLS) != PackageManager.PERMISSION_GRANTED) return
        val telecom = context.getSystemService(Context.TELECOM_SERVICE) as TelecomManager
        runCatching { telecom.endCall() }
    }
}
