package com.phonebridge.app.network

import com.phonebridge.app.media.MediaControllerBridge
import com.phonebridge.app.sms.SmsBridge
import org.java_websocket.client.WebSocketClient
import org.java_websocket.handshake.ServerHandshake
import org.json.JSONObject
import java.net.URI

class SignalingClient(
    private val onCommand: (String, Map<String, String>) -> Unit = { type, data ->
        when {
            type.startsWith("media_") -> MediaControllerBridge.handleCommand(type)
            type == "sms_send" -> SmsBridge.sendFromCommand(data)
            type == "sms_list" -> SmsBridge.publishRecent()
        }
    }
) {

    private var client: WebSocketClient? = null

    fun connect(url: String) {
        client = createClient(url)
        client?.connect()
    }

    /** Used by SMS_RECEIVED when the normal bridge session is not running. */
    fun connectBlocking(url: String, timeoutMs: Long = 3000): Boolean {
        client = createClient(url)
        return try {
            client?.connectBlocking(timeoutMs, java.util.concurrent.TimeUnit.MILLISECONDS) == true
        } catch (_: InterruptedException) {
            Thread.currentThread().interrupt()
            false
        }
    }

    private fun createClient(url: String): WebSocketClient = object : WebSocketClient(URI(url)) {
        override fun onOpen(handshakedata: ServerHandshake?) {
            send("{\"type\":\"register\",\"device\":\"android\",\"capabilities\":[\"calls\",\"media\",\"sms\"]}")
        }

        override fun onMessage(message: String?) {
            if (message.isNullOrBlank()) return
            try {
                val root = JSONObject(message)
                val type = root.optString("type")
                if (type.isBlank()) return

                val data = mutableMapOf<String, String>()
                val payload = root.optJSONObject("data")
                if (payload != null) {
                    payload.keys().forEach { key ->
                        data[key] = payload.optString(key)
                    }
                }
                onCommand(type, data)

                if (type.startsWith("media_")) {
                    sendEvent("media_state", MediaControllerBridge.snapshot())
                }
            } catch (_: Exception) {
                // Ignore malformed or unsupported control messages.
            }
        }

        override fun onClose(code: Int, reason: String?, remote: Boolean) {}
        override fun onError(ex: Exception?) {}
    }

    fun sendEvent(type: String, data: Map<String, String>) {
        val root = JSONObject().apply {
            put("type", type)
            put("data", JSONObject().apply {
                data.forEach { (key, value) -> put(key, value) }
            })
        }
        client?.takeIf { it.isOpen }?.send(root.toString())
    }

    fun disconnect() {
        client?.close()
        client = null
    }

    companion object {
        const val DEFAULT_URL = "ws://192.168.137.1:5000"
    }
}
