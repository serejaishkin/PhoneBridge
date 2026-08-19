package com.phonebridge2.app.discovery

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import java.net.DatagramPacket
import java.net.DatagramSocket
import java.net.InetSocketAddress

/** Зеркало discovery::Announce на PC-стороне. Поля и имена — 1 в 1. */
@Serializable
data class Announce(
    val device_id: String,
    val device_name: String,
    val platform: String,
    val pairing_port: Int,
    val fingerprint: String,
)

object DiscoveryClient {
    const val DISCOVERY_PORT = 17592
    private val json = Json { ignoreUnknownKeys = true }

    fun listen(scope: CoroutineScope, onFound: (Announce, String) -> Unit) {
        scope.launch(Dispatchers.IO) {
            DatagramSocket(null).use { socket ->
                socket.reuseAddress = true
                socket.bind(InetSocketAddress(DISCOVERY_PORT))
                val buf = ByteArray(2048)
                while (true) {
                    val packet = DatagramPacket(buf, buf.size)
                    socket.receive(packet)
                    val text = String(packet.data, 0, packet.length, Charsets.UTF_8)
                    runCatching { json.decodeFromString<Announce>(text) }
                        .onSuccess { announce ->
                            onFound(announce, packet.address.hostAddress ?: "")
                        }
                        .onFailure {
                            // Broadcast may contain unrelated traffic; ignore it.
                        }
                }
            }
        }
    }
}
