package com.phonebridge.app.network

import android.os.Build
import com.phonebridge.app.media.MediaControllerBridge
import org.json.JSONObject
import java.io.BufferedReader
import java.io.BufferedWriter
import java.io.InputStreamReader
import java.io.OutputStreamWriter
import java.net.InetSocketAddress
import java.net.Socket
import java.util.UUID
import javax.net.ssl.SSLContext
import javax.net.ssl.SSLSocket
import javax.net.ssl.X509TrustManager

/** TLS + newline-delimited JSON control client matching pc/src/protocol.rs. */
class SignalingClient(
    private val onCommand: (String, Map<String, String>) -> Unit = { _, _ -> }
) {
    @Volatile private var socket: SSLSocket? = null
    @Volatile private var writer: BufferedWriter? = null
    @Volatile private var connected = false

    fun connect(url: String) { Thread { runCatching { open(url, 5_000) } }.start() }

    fun connectBlocking(url: String, timeoutMs: Long = 5_000): Boolean = try {
        open(url, timeoutMs); connected
    } catch (_: Exception) {
        disconnect(); false
    }

    private fun open(url: String, timeoutMs: Long) {
        disconnect()
        val (host, port) = parseTarget(url)
        val trustAll = object : X509TrustManager {
            override fun getAcceptedIssuers() = emptyArray<java.security.cert.X509Certificate>()
            override fun checkClientTrusted(chain: Array<java.security.cert.X509Certificate>, authType: String) = Unit
            override fun checkServerTrusted(chain: Array<java.security.cert.X509Certificate>, authType: String) = Unit
        }
        val context = SSLContext.getInstance("TLS").apply {
            init(null, arrayOf(trustAll), java.security.SecureRandom())
        }
        val raw = Socket().apply { connect(InetSocketAddress(host, port), timeoutMs.toInt()) }
        val ssl = context.socketFactory.createSocket(raw, host, port, true) as SSLSocket
        ssl.startHandshake()
        socket = ssl
        writer = BufferedWriter(OutputStreamWriter(ssl.outputStream, Charsets.UTF_8))
        connected = true
        sendJson(hello())

        Thread {
            try {
                BufferedReader(InputStreamReader(ssl.inputStream, Charsets.UTF_8)).use { reader ->
                    while (connected) {
                        val line = reader.readLine() ?: break
                        handleLine(line)
                    }
                }
            } catch (_: Exception) {
                // Normal when the socket is closed.
            } finally { connected = false }
        }.apply { name = "PhoneBridge-ControlReader"; isDaemon = true; start() }
    }

    private fun handleLine(line: String) {
        if (line.isBlank()) return
        runCatching {
            val root = JSONObject(line)
            val type = root.optString("type")
            val data = root.optJSONObject("data")
            when (type) {
                "HelloAck" -> sendEvent("media_state", MediaControllerBridge.snapshot())
                "Ping" -> sendJson(JSONObject().put("type", "Pong"))
                "MediaCommand" -> {
                    onCommand("media_command", mapOf("command" to data?.optString("command").orEmpty()))
                    sendEvent("media_state", MediaControllerBridge.snapshot())
                }
                "CallAnswer" -> onCommand("call_answer", emptyMap())
                "CallDecline" -> onCommand("call_decline", emptyMap())
                "sms_send" -> onCommand("sms_send", mapOf("address" to data?.optString("address").orEmpty(), "body" to data?.optString("body").orEmpty()))
                "sms_list" -> onCommand("sms_list", emptyMap())
                "PcBluetoothStatus" -> onCommand("pc_bluetooth_status", mapOf("hfp_supported" to data?.optString("hfp_supported").orEmpty()))
            }
        }
    }

    fun sendEvent(type: String, data: Map<String, String>) {
        val message = when (type) {
            "incoming_call" -> JSONObject().put("type", "IncomingCall").put("data", JSONObject().apply { put("caller_number", data["number"]); put("caller_name", data["name"]) })
            "call_ended" -> JSONObject().put("type", "CallEnded")
            "media_state" -> JSONObject().put("type", "MediaState").put("data", JSONObject().apply {
                put("package", data["package"]); put("state", data["state"].orEmpty().replaceFirstChar { it.uppercase() }); put("title", data["title"]); put("artist", data["artist"]); put("album", data["album"])
            })
            "sms_received" -> JSONObject().put("type", "sms_received").put("data", JSONObject().apply { put("address", data["address"].orEmpty()); put("body", data["body"].orEmpty()); put("timestamp", data["timestamp"]?.toLongOrNull() ?: 0L) })
            "sms_item" -> JSONObject().put("type", "sms_item").put("data", JSONObject().apply { put("id", data["id"].orEmpty()); put("address", data["address"].orEmpty()); put("body", data["body"].orEmpty()); put("timestamp", data["timestamp"]?.toLongOrNull() ?: 0L) })
            "sms_list_end" -> JSONObject().put("type", "sms_list_end").put("data", JSONObject().put("count", data["count"]?.toIntOrNull() ?: 0))
            "sms_sent" -> JSONObject().put("type", "sms_sent").put("data", JSONObject().put("address", data["address"].orEmpty()).put("body", data["body"].orEmpty()))
            "sms_error" -> JSONObject().put("type", "sms_error").put("data", JSONObject().put("error", data["error"].orEmpty()))
            else -> null
        }
        if (message != null) sendJson(message)
    }

    private fun hello() = JSONObject().put("type", "Hello").put("data", JSONObject().apply {
        put("device_id", UUID.nameUUIDFromBytes((Build.BRAND + ":" + Build.DEVICE).toByteArray()).toString())
        put("device_name", Build.MODEL); put("platform", "android"); put("protocol_version", 1)
    })

    private fun sendJson(json: JSONObject) {
        synchronized(this) {
            if (!connected) return
            runCatching { writer?.apply { write(json.toString()); newLine(); flush() } }.onFailure { connected = false }
        }
    }

    fun disconnect() {
        connected = false
        runCatching { writer?.close() }; runCatching { socket?.close() }
        writer = null; socket = null
    }

    fun isConnected() = connected

    private fun parseTarget(url: String): Pair<String, Int> {
        val normalized = url.removePrefix("tls://").removePrefix("tcp://").removePrefix("ws://")
        val host = normalized.substringBefore(":").ifBlank { "192.168.137.1" }
        val port = normalized.substringAfter(":", "17591").toIntOrNull() ?: 17591
        return host to port
    }

    companion object { const val DEFAULT_URL = "tls://192.168.137.1:17591" }
}
