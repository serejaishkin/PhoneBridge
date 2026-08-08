package com.phonebridge.app.network

import org.java_websocket.client.WebSocketClient
import org.java_websocket.handshake.ServerHandshake
import java.net.URI

class SignalingClient {

    private var client: WebSocketClient? = null

    fun connect(url: String) {
        client = object : WebSocketClient(URI(url)) {
            override fun onOpen(handshakedata: ServerHandshake?) {
                send("{\"type\":\"register\",\"device\":\"android\"}")
            }

            override fun onMessage(message: String?) {
                // Handle commands from PC (answer_call, end_call)
            }

            override fun onClose(code: Int, reason: String?, remote: Boolean) {}
            override fun onError(ex: Exception?) {}
        }
        client?.connect()
    }

    fun sendEvent(type: String, data: Map<String, String>) {
        val json = buildString {
            append("{\"type\":\"$type\",\"data\":{")
            data.entries.joinTo(this, ",") { "\"${it.key}\":\"${it.value}\"" }
            append("}}")
        }
        client?.send(json)
    }

    fun disconnect() {
        client?.close()
    }
}
