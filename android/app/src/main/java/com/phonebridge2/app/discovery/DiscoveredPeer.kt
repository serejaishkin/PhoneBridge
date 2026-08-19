package com.phonebridge2.app.discovery

/** Immutable discovery record. Discovery is location, not authentication. */
data class DiscoveredPeer(
    val deviceId: String,
    val deviceName: String,
    val platform: String,
    val host: String,
    val port: Int,
    val fingerprint: String,
    val discoveredAtMs: Long = System.currentTimeMillis(),
)

fun DiscoveredPeer.isExpired(nowMs: Long = System.currentTimeMillis(), ttlMs: Long = 10_000L): Boolean =
    nowMs - discoveredAtMs > ttlMs
