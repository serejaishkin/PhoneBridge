package com.phonebridge.app.pairing

import android.content.Context
import java.io.File
import org.json.JSONObject

/**
 * Phone-side pin store for PC certificate fingerprints, mirroring
 * pc/src/pairing/trust.rs semantics (trust-on-first-use + explicit pins).
 *
 * Pins are keyed either by host ("host:<ip>") to validate the TLS server
 * certificate during the handshake, or by device id ("device:<id>") once the
 * HelloAck confirms which PC answered. All mutations are persisted as JSON.
 */
class TrustStore private constructor(private val backingFile: File) {

    private val pins = LinkedHashMap<String, String>()

    companion object {
        private const val FILE_NAME = "pc_pins.json"

        fun load(context: Context): TrustStore {
            val dir = File(context.filesDir, "pairing").apply { mkdirs() }
            val target = File(dir, FILE_NAME)
            val store = TrustStore(target)
            if (target.exists()) {
                runCatching {
                    val root = JSONObject(target.readText())
                    for (key in root.keys()) store.pins[key] = root.optString(key)
                }
            }
            return store
        }
    }

    @Synchronized
    fun pinFor(key: String): String? = pins[key]

    @Synchronized
    fun isTrusted(key: String, fingerprintHex: String): Boolean =
        pins[key]?.equals(fingerprintHex, ignoreCase = true) == true

    @Synchronized
    fun pin(key: String, fingerprintHex: String) {
        if (isTrusted(key, fingerprintHex)) return
        pins[key] = fingerprintHex.lowercase()
        save()
    }

    @Synchronized
    fun revoke(key: String) {
        if (pins.remove(key) != null) save()
    }

    private fun save() {
        runCatching {
            val root = JSONObject()
            for ((k, v) in pins) root.put(k, v)
            backingFile.parentFile?.mkdirs()
            backingFile.writeText(root.toString(2))
        }
    }
}
