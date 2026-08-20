package com.phonebridge2.app.connection

/**
 * Ordered reconnect routes. Wi-Fi and PC hotspot are both TCP/IP routes;
 * Bluetooth PAN is also represented as an IP route when the OS exposes the
 * Bluetooth network adapter. Direct Bluetooth RFCOMM is deliberately a
 * separate future transport because TLS must run over the same byte stream.
 */
data class ConnectionRoute(
    val host: String,
    val port: Int,
    val fingerprint: String,
    val kind: Kind,
    val priority: Int = 0,
) {
    enum class Kind { WIFI, HOTSPOT, BLUETOOTH_PAN, MANUAL }
}

class RouteSet(routes: List<ConnectionRoute>) {
    val routes: List<ConnectionRoute> = routes.distinctBy { "${it.kind}:${it.host}:${it.port}" }
        .sortedBy { it.priority }

    fun nextAfter(kind: ConnectionRoute.Kind): List<ConnectionRoute> =
        routes.sortedBy { if (it.kind == kind) 0 else 1 }
}
