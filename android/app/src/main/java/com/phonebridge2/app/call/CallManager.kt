package com.phonebridge2.app.call

import android.content.Context
import android.telephony.TelephonyCallback
import android.telephony.TelephonyManager
import androidx.core.content.ContextCompat
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow

sealed class CallState {
    object Idle : CallState()
    data class Ringing(val number: String?) : CallState()
    object Active : CallState()
}

/**
 * Замена deprecated PhoneStateListener из PhoneBridge v1 (см. историю проекта —
 * там это было явно отмечено как технический долг). TelephonyCallback доступен
 * с API 31, поэтому minSdk этого модуля = 31 (см. build.gradle.kts).
 *
 * ВАЖНО: этот класс отвечает ТОЛЬКО за наблюдение за состоянием звонка и номером
 * (для отображения на ПК). Управление звонком (answer/decline) — в
 * BridgeInCallService.kt, это разные Android API и разные обязанности.
 */
class CallManager(private val context: Context) {

    private val _state = MutableStateFlow<CallState>(CallState.Idle)
    val state: StateFlow<CallState> = _state

    private val telephonyManager =
        context.getSystemService(Context.TELEPHONY_SERVICE) as TelephonyManager

    private var callback: TelephonyCallback? = null

    fun start() {
        // READ_PHONE_STATE нужен именно для CallStateListener с номером звонящего;
        // без него callback всё равно сработает, но number будет всегда null —
        // это осознанное поведение Android, не баг.
        val hasReadPhoneState = ContextCompat.checkSelfPermission(
            context, android.Manifest.permission.READ_PHONE_STATE
        ) == android.content.pm.PackageManager.PERMISSION_GRANTED

        val cb = object : TelephonyCallback(), TelephonyCallback.CallStateListener {
            override fun onCallStateChanged(state: Int) {
                _state.value = when (state) {
                    TelephonyManager.CALL_STATE_RINGING -> CallState.Ringing(number = null)
                    TelephonyManager.CALL_STATE_OFFHOOK -> CallState.Active
                    else -> CallState.Idle
                }
            }
        }
        callback = cb

        if (!hasReadPhoneState) {
            // Не бросаем исключение — просто работаем без номера. UI должен
            // отдельно объяснить пользователю, зачем READ_PHONE_STATE вообще нужен
            // (см. AI_HANDOFF_GUI.md, онбординг — "объяснить каждое разрешение").
        }
        telephonyManager.registerTelephonyCallback(context.mainExecutor, cb)
    }

    fun stop() {
        callback?.let { telephonyManager.unregisterTelephonyCallback(it) }
        callback = null
    }
}
