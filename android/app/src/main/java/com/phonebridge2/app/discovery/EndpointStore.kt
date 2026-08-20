package com.phonebridge2.app.discovery

import android.content.Context
import org.json.JSONObject

/** Persists the last trusted PC endpoints so reconnect does not depend on discovery. */
class EndpointStore(context: Context) {
    private val prefs = context.getSharedPreferences("phonebridge_endpoints", Context.MODE_PRIVATE)

    fun save(peer: DiscoveredPeer) {
        prefs.edit().putString(peer.deviceId, JSONObject().apply {
            put("deviceId", peer.deviceId)
            put("deviceName", peer.deviceName)
            put("platform", peer.platform)
            put("host", peer.host)
            put("port", peer.port)
            put("fingerprint", peer.fingerprint)
            put("discoveredAtMs", peer.discoveredAtMs)
        }.toString()).apply()
    }

    fun load(deviceId: String): DiscoveredPeer? = prefs.getString(deviceId, null)?.let { raw ->
        val json = JSONObject(raw)
        DiscoveredPeer(
            deviceId = json.getString("deviceId"),
            deviceName = json.getString("deviceName"),
            platform = json.getString("platform"),
            host = json.getString("host"),
            port = json.getInt("port"),
            fingerprint = json.getString("fingerprint"),
            discoveredAtMs = json.optLong("discoveredAtMs", 0L),
        )
    }

    fun remove(deviceId: String) { prefs.edit().remove(deviceId).apply() }
}
