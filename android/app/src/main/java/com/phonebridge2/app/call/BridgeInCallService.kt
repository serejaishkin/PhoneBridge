package com.phonebridge2.app.call

import android.telecom.Call
import android.telecom.InCallService
import android.util.Log

/**
 * В PhoneBridge v1 (см. историю проекта) answerCall()/endCall() были пустыми
 * комментариями — это и было главной находкой ревью. Здесь — рабочая
 * реализация через настоящий Android Telecom API.
 *
 * Регистрация как InCallService (см. AndroidManifest.xml) даёт доступ к
 * управлению звонком БЕЗ необходимости быть default dialer — этого достаточно
 * для answer/decline, дефолтным дозвонщиком становиться не нужно.
 */
class BridgeInCallService : InCallService() {

    private var currentCall: Call? = null

    private val callCallback = object : Call.Callback() {
        override fun onStateChanged(call: Call, state: Int) {
            Log.d(TAG, "call state changed: $state")
            if (state == Call.STATE_DISCONNECTED) {
                currentCall = null
            }
        }
    }

    override fun onCallAdded(call: Call) {
        currentCall = call
        call.registerCallback(callCallback)
        Log.d(TAG, "call added, state=${call.state}")
        // TODO: отправить Message.IncomingCall на PC через уже установленное
        // pairing-соединение (см. pairing/ — клиентская часть TLS-соединения
        // пока не написана в этом скелете, см. HANDOFF-документ "что дальше").
    }

    override fun onCallRemoved(call: Call) {
        call.unregisterCallback(callCallback)
        if (currentCall == call) currentCall = null
        Log.d(TAG, "call removed")
        // TODO: отправить Message.CallEnded на PC
    }

    /** Вызывается, когда с PC пришла команда Message.CallAnswer. */
    fun answerCurrentCall() {
        currentCall?.answer(android.telecom.VideoProfile.STATE_AUDIO_ONLY)
    }

    /** Вызывается, когда с PC пришла команда Message.CallDecline. */
    fun declineCurrentCall() {
        currentCall?.reject(false, null)
    }

    companion object {
        private const val TAG = "BridgeInCallService"
    }
}
