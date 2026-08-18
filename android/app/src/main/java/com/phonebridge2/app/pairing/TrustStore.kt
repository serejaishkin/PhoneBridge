package com.phonebridge2.app.pairing

import android.content.Context
import org.json.JSONObject

/**
 * Аналог pc/src/pairing/trust.rs. Хранит device_id -> fingerprint_hex в обычном
 * приватном файле приложения (не SharedPreferences, чтобы формат было легко
 * сверить/задампить руками при отладке пейринга).
 */
class TrustStore(context: Context) {
    private val file = java.io.File(context.filesDir, "trusted_peers.json")
    private val peers: MutableMap<String, String> = mutableMapOf()

    init {
        if (file.exists()) {
            val json = JSONObject(file.readText())
            json.keys().forEach { key -> peers[key] = json.getString(key) }
        }
    }

    fun isTrusted(deviceId: String, fingerprintHex: String): Boolean =
        peers[deviceId] == fingerprintHex

    fun trust(deviceId: String, fingerprintHex: String) {
        peers[deviceId] = fingerprintHex
        save()
    }

    fun revoke(deviceId: String) {
        peers.remove(deviceId)
        save()
    }

    private fun save() {
        val json = JSONObject()
        peers.forEach { (k, v) -> json.put(k, v) }
        file.writeText(json.toString())
    }

    companion object {
        /**
         * ДОЛЖЕН давать идентичный результат pairing::trust::short_code() на PC-стороне —
         * первые 8 hex-символов SHA-256 отпечатка, сгруппированные "XXXX-XXXX".
         * Если меняешь один — меняй оба, иначе пользователь будет сверять разные коды.
         */
        fun shortCode(fingerprintHex: String): String {
            val upper = fingerprintHex.uppercase()
            val chunk = upper.take(8)
            return "${chunk.substring(0, 4)}-${chunk.substring(4, 8)}"
        }
    }
}
