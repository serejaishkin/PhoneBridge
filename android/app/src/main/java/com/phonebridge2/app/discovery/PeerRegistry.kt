package com.phonebridge2.app.discovery

import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow

class PeerRegistry {
    private val peers = LinkedHashMap<String, DiscoveredPeer>()
    private val _state = MutableStateFlow<List<DiscoveredPeer>>(emptyList())
    val state: StateFlow<List<DiscoveredPeer>> = _state.asStateFlow()

    @Synchronized
    fun upsert(peer: DiscoveredPeer) {
        peers[peer.deviceId] = peer
        publish()
    }

    @Synchronized
    fun remove(deviceId: String) {
        peers.remove(deviceId)
        publish()
    }

    @Synchronized
    fun prune(nowMs: Long = System.currentTimeMillis()) {
        peers.entries.removeIf { it.value.isExpired(nowMs) }
        publish()
    }

    @Synchronized
    fun get(deviceId: String): DiscoveredPeer? = peers[deviceId]

    private fun publish() {
        _state.value = peers.values.sortedBy { it.deviceName.lowercase() }
    }
}
