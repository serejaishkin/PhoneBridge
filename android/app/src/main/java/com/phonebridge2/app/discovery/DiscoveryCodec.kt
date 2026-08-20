package com.phonebridge2.app.discovery

import org.json.JSONObject

/** UDP discovery payload codec. Keep this deliberately separate from TLS protocol. */
object DiscoveryCodec {
    private const val VERSION = 1

    fun encode(peer: DiscoveredPeer): ByteArray = JSONObject()
        .put("version", VERSION)
        .put("device_id", peer.deviceId)
        .put("device_name", peer.deviceName)
        .put("platform", peer.platform)
        .put("port", peer.port)
        .put("fingerprint", peer.fingerprint)
        .toString()
        .toByteArray(Charsets.UTF_8)

    fun decode(payload: ByteArray, sourceHost: String, nowMs: Long = System.currentTimeMillis()): DiscoveredPeer? = runCatching {
        val json = JSONObject(String(payload, Charsets.UTF_8))
        require(json.getInt("version") == VERSION)
        val deviceId = json.getString("device_id")
        val name = json.getString("device_name")
        val platform = json.getString("platform")
        val port = json.getInt("port")
        val fingerprint = json.getString("fingerprint").uppercase()
        require(deviceId.isNotBlank())
        require(fingerprint.length >= 16)
        require(port in 1..65535)
        DiscoveredPeer(deviceId, name, platform, sourceHost, port, fingerprint, nowMs)
    }.getOrNull()
}
