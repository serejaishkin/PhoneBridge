package com.phonebridge.app.network

import android.os.Build
import com.phonebridge.app.media.MediaControllerBridge
import com.phonebridge.app.pairing.PhoneIdentity
import com.phonebridge.app.pairing.TrustStore
import java.io.BufferedReader
import java.io.BufferedWriter
import java.io.InputStreamReader
import java.io.OutputStreamWriter
import java.net.InetSocketAddress
import java.net.Socket
import java.security.MessageDigest
import java.security.Principal
import java.security.PrivateKey
import java.security.SecureRandom
import java.security.cert.X509Certificate
import java.util.UUID
import javax.net.ssl.KeyManager
import javax.net.ssl.SSLContext
import javax.net.ssl.SSLSocket
import javax.net.ssl.X509ExtendedKeyManager
import javax.net.ssl.X509TrustManager
import org.json.JSONObject

/** TLS control client. Comments intentionally remain in English for cross-platform maintenance. */
class SignalingClient(
    private val onCommand: (String, Map<String, String>) -> Unit = { _, _ -> },
    private val onStatus: (String, String?) -> Unit = { _, _ -> },
    private val identityProvider: () -> PhoneIdentity? = { null },
    private val trustProvider: () -> TrustStore? = { null }
) {
    @Volatile private var socket: SSLSocket? = null
    @Volatile private var writer: BufferedWriter? = null
    @Volatile private var connected = false
    @Volatile private var currentHost: String? = null
    @Volatile private var firstConnection = false

    private class IdentityKeyManager(private val identity: PhoneIdentity) : X509ExtendedKeyManager() {
        companion object { const val ALIAS = "phonebridge-identity" }
        override fun getClientAliases(keyType: String?, issuers: Array<out Principal>?) = if (keyType == "RSA") arrayOf(ALIAS) else emptyArray<String>()
        override fun chooseClientAlias(keyType: Array<out String>?, issuers: Array<out Principal>?, socket: Socket?): String? = if (keyType?.contains("RSA") == true) ALIAS else null
        override fun getCertificateChain(alias: String?): Array<X509Certificate>? = if (alias == ALIAS) identity.certificateChain() else null
        override fun getPrivateKey(alias: String?): PrivateKey? = if (alias == ALIAS) identity.privateKey() else null
        override fun getServerAliases(keyType: String?, issuers: Array<out Principal>?) = null
        override fun chooseServerAlias(keyType: String?, issuers: Array<out Principal>?, socket: Socket?) = null
    }

    fun connect(url: String) { Thread { runCatching { open(url, 5_000) } }.start() }
    fun connectBlocking(url: String, timeoutMs: Long = 5_000): Boolean = try { open(url, timeoutMs); connected } catch (_: Exception) { disconnect(); false }

    private fun open(url: String, timeoutMs: Long) {
        disconnect()
        val (host, port) = parseTarget(url)
        currentHost = host
        val trustStore = trustProvider()
        val identity = identityProvider()
        val expectedFingerprint = trustStore?.pinFor("host:$host")
        firstConnection = false
        var recordedPin: String? = null

        val pinningTrustManager = object : X509TrustManager {
            override fun getAcceptedIssuers(): Array<X509Certificate> = emptyArray()
            override fun checkClientTrusted(chain: Array<X509Certificate>, authType: String) = Unit
            override fun checkServerTrusted(chain: Array<X509Certificate>, authType: String) {
                val leaf = chain.firstOrNull() ?: throw java.security.cert.CertificateException("empty server chain")
                val fingerprint = sha256Hex(leaf.encoded)
                when {
                    expectedFingerprint == null -> { firstConnection = true; recordedPin = fingerprint }
                    !fingerprint.equals(expectedFingerprint, ignoreCase = true) -> throw java.security.cert.CertificateException("PC certificate changed for $host")
                }
            }
        }

        val context = SSLContext.getInstance("TLS").apply {
            val keyManagers: Array<KeyManager>? = identity?.let { arrayOf<KeyManager>(IdentityKeyManager(it)) }
            init(keyManagers, arrayOf(pinningTrustManager), SecureRandom())
        }
        val raw = Socket().apply { connect(InetSocketAddress(host, port), timeoutMs.toInt()) }
        val ssl = context.socketFactory.createSocket(raw, host, port, true) as SSLSocket
        ssl.startHandshake()
        if (firstConnection && trustStore != null && recordedPin != null) { trustStore.pin("host:$host", recordedPin!!); onStatus("first_connection", host) }
        socket = ssl
        writer = BufferedWriter(OutputStreamWriter(ssl.outputStream, Charsets.UTF_8))
        connected = true
        sendJson(hello())
        Thread {
            try {
                BufferedReader(InputStreamReader(ssl.inputStream, Charsets.UTF_8)).use { reader ->
                    while (connected) { val line = reader.readLine() ?: break; handleLine(line) }
                }
            } catch (_: Exception) {
                // Socket closure is handled by the connection state.
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
                "HelloAck" -> {
                    val trusted = data?.optBoolean("trusted") ?: false
                    val pcFingerprint = data?.optString("cert_fingerprint").orEmpty()
                    val pcDeviceId = data?.optString("device_id").orEmpty()
                    val host = currentHost
                    val trustStore = trustProvider()
                    if (pcFingerprint.isNotEmpty() && host != null && trustStore != null) {
                        val pinned = trustStore.pinFor("host:$host")
                        if (pinned != null && !pcFingerprint.equals(pinned, ignoreCase = true)) { onStatus("certificate_mismatch", host); disconnect(); return }
                        if (pcDeviceId.isNotEmpty()) trustStore.pin("device:$pcDeviceId", pcFingerprint)
                    }
                    if (!trusted) { onStatus("pairing_required", host); return }
                    onStatus("connected", null)
                    sendEvent("media_state", MediaControllerBridge.snapshot())
                }
                "PairChallenge" -> {
                    val code = data?.optString("short_code").orEmpty()
                    val deviceId = data?.optString("device_id").orEmpty()
                    onStatus("pairing_challenge", "$deviceId:$code")
                }
                "PairResult" -> {
                    val ok = data?.optBoolean("trusted") ?: false
                    onStatus(if (ok) "paired" else "pairing_rejected", currentHost)
                }
                "Ping" -> sendJson(JSONObject().put("type", "Pong"))
                "MediaCommand" -> { onCommand("media_command", mapOf("command" to data?.optString("command").orEmpty())); sendEvent("media_state", MediaControllerBridge.snapshot()) }
                "CallAnswer" -> onCommand("call_answer", emptyMap())
                "CallDecline" -> onCommand("call_decline", emptyMap())
                "sms_send" -> onCommand("sms_send", mapOf("address" to data?.optString("address").orEmpty(), "body" to data?.optString("body").orEmpty()))
                "sms_list" -> onCommand("sms_list", emptyMap())
                "PcBluetoothStatus" -> onCommand("pc_bluetooth_status", mapOf("hfp_supported" to data?.optString("hfp_supported").orEmpty()))
                "Error" -> onStatus("error", data?.optString("message"))
            }
        }
    }

    fun confirmPairing(deviceId: String, shortCode: String) = sendJson(JSONObject().put("type", "PairConfirm").put("data", JSONObject().put("device_id", deviceId).put("short_code", shortCode)))

    fun sendEvent(type: String, data: Map<String, String>) {
        val message = when (type) {
            "incoming_call" -> JSONObject().put("type", "IncomingCall").put("data", JSONObject().apply { put("caller_number", data["number"]); put("caller_name", data["name"]) })
            "call_ended" -> JSONObject().put("type", "CallEnded")
            "call_active" -> JSONObject().put("type", "CallActive")
            "mic_start" -> JSONObject().put("type", "MicStart")
            "mic_stop" -> JSONObject().put("type", "MicStop")
            "media_state" -> JSONObject().put("type", "MediaState").put("data", JSONObject().apply { put("package", data["package"]); put("state", data["state"].orEmpty().replaceFirstChar { it.uppercase() }); put("title", data["title"]); put("artist", data["artist"]); put("album", data["album"]) })
            "sms_received" -> JSONObject().put("type", "sms_received").put("data", JSONObject().apply { put("address", data["address"].orEmpty()); put("body", data["body"].orEmpty()); put("timestamp", data["timestamp"]?.toLongOrNull() ?: 0L) })
            "sms_item" -> JSONObject().put("type", "sms_item").put("data", JSONObject().apply { put("id", data["id"].orEmpty()); put("address", data["address"].orEmpty()); put("body", data["body"].orEmpty()); put("timestamp", data["timestamp"]?.toLongOrNull() ?: 0L) })
            "sms_list_end" -> JSONObject().put("type", "sms_list_end").put("data", JSONObject().put("count", data["count"]?.toIntOrNull() ?: 0))
            "sms_sent" -> JSONObject().put("type", "sms_sent").put("data", JSONObject().put("address", data["address"].orEmpty()).put("body", data["body"].orEmpty()))
            "sms_error" -> JSONObject().put("type", "sms_error").put("data", JSONObject().put("error", data["error"].orEmpty()))
            else -> null
        }
        if (message != null) sendJson(message)
    }

    private fun hello(): JSONObject {
        val identity = identityProvider()
        return JSONObject().put("type", "Hello").put("data", JSONObject().apply {
            if (identity != null) { put("device_id", identity.deviceId); put("cert_fingerprint", identity.fingerprintHex()) }
            else put("device_id", UUID.nameUUIDFromBytes((Build.BRAND + ":" + Build.DEVICE).toByteArray()).toString())
            put("device_name", Build.MODEL); put("platform", "android"); put("protocol_version", 1)
        })
    }

    private fun sha256Hex(bytes: ByteArray): String = MessageDigest.getInstance("SHA-256").digest(bytes).joinToString("") { "%02x".format(it) }
    private fun sendJson(json: JSONObject) { synchronized(this) { if (!connected) return; runCatching { writer?.apply { write(json.toString()); newLine(); flush() } }.onFailure { connected = false } } }
    fun disconnect() { connected = false; runCatching { writer?.close() }; runCatching { socket?.close() }; writer = null; socket = null }
    fun isConnected() = connected

    private fun parseTarget(url: String): Pair<String, Int> {
        val normalized = url.removePrefix("tls://").removePrefix("tcp://").removePrefix("ws://")
        val host = normalized.substringBefore(":").ifBlank { "192.168.137.1" }
        val port = normalized.substringAfter(":", "17591").toIntOrNull() ?: 17591
        return host to port
    }

    companion object { const val DEFAULT_URL = "tls://192.168.137.1:17591" }
}
