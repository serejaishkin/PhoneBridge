package com.phonebridge2.app.discovery

enum class TransportKind { WIFI, HOTSPOT, BLUETOOTH_PAN, MANUAL }

data class ConnectionRoute(
    val kind: TransportKind,
    val host: String,
    val port: Int,
    val fingerprint: String,
    val priority: Int,
)

/** Produces deterministic reconnect order without treating discovery as trust. */
object RoutePlanner {
    fun plan(peer: DiscoveredPeer, manualHost: String? = null): List<ConnectionRoute> = buildList {
        add(ConnectionRoute(TransportKind.WIFI, peer.host, peer.port, peer.fingerprint, 10))
        add(ConnectionRoute(TransportKind.HOTSPOT, peer.host, peer.port, peer.fingerprint, 20))
        add(ConnectionRoute(TransportKind.BLUETOOTH_PAN, peer.host, peer.port, peer.fingerprint, 30))
        if (!manualHost.isNullOrBlank()) add(ConnectionRoute(TransportKind.MANUAL, manualHost, peer.port, peer.fingerprint, 40))
    }.sortedBy { it.priority }
}
