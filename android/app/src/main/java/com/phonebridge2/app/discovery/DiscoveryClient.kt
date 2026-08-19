package com.phonebridge2.app.discovery

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import java.net.DatagramPacket
import java.net.DatagramSocket
import java.net.InetSocketAddress

/** UDP discovery is deliberately independent from TLS authentication. */
@Serializable
data class Announce(
    val device_id: String,
    val device_name: String,
    val platform: String,
    val pairing_port: Int,
    val fingerprint: String,
)

class DiscoveryClient(
    private val scope: CoroutineScope,
    private val discoveryPort: Int = 17592,
) {
    private val json = Json { ignoreUnknownKeys = true }
    private val _peers = MutableStateFlow<List<DiscoveredPeer>>(emptyList())
    val peers: StateFlow<List<DiscoveredPeer>> = _peers.asStateFlow()
    private var job: Job? = null

    fun start() {
        if (job?.isActive == true) return
        job = scope.launch(Dispatchers.IO) {
            DatagramSocket(null).use { socket ->
                socket.reuseAddress = true
                socket.bind(InetSocketAddress(discoveryPort))
                socket.soTimeout = 1000
                val buf = ByteArray(4096)
                while (isActive) {
                    try {
                        val packet = DatagramPacket(buf, buf.size)
                        socket.receive(packet)
                        val text = String(packet.data, 0, packet.length, Charsets.UTF_8)
                        val announce = runCatching { json.decodeFromString<Announce>(text) }.getOrNull() ?: continue
                        val host = packet.address.hostAddress ?: continue
                        val peer = DiscoveredPeer(
                            deviceId = announce.device_id,
                            deviceName = announce.device_name,
                            platform = announce.platform,
                            host = host,
                            port = announce.pairing_port,
                            fingerprint = announce.fingerprint.uppercase(),
                        )
                        _peers.value = (_peers.value.filterNot { it.deviceId == peer.deviceId } + peer)
                            .filterNot { it.isExpired() }
                    } catch (_: java.net.SocketTimeoutException) {
                        _peers.value = _peers.value.filterNot { it.isExpired() }
                    }
                }
            }
        }
    }

    fun stop() {
        job?.cancel()
        job = null
        _peers.value = emptyList()
    }
}
