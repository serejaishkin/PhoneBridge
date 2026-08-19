package com.phonebridge2.app.connection

data class Peer(
    val deviceId: String,
    val deviceName: String,
    val host: String,
    val port: Int,
    val fingerprint: String,
)
