package com.phonebridge2.app.pairing

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.io.BufferedInputStream
import java.io.BufferedOutputStream
import java.net.InetSocketAddress
import java.security.SecureRandom
import java.security.cert.X509Certificate
import java.util.UUID
import javax.net.ssl.SSLContext
import javax.net.ssl.SSLSocket
import javax.net.ssl.TrustManager
import javax.net.ssl.X509TrustManager

/**
 * Minimal Android client for the first end-to-end PC pairing test.
 *
 * Discovery is intentionally kept outside this class: the caller supplies
 * the PC address discovered by Wi-Fi, hotspot, or later Bluetooth adapters.
 */
class PcPairingClient(
    private val port: Int = 1716,
) {
    suspend fun pair(host: String): PairingResult = withContext(Dispatchers.IO) {
        try {
            createTestTlsContext().socketFactory.createSocket().use { raw ->
                val socket = raw as SSLSocket
                socket.connect(InetSocketAddress(host, port), 5_000)
                socket.soTimeout = 10_000
                socket.startHandshake()

                val input = BufferedInputStream(socket.inputStream)
                val output = BufferedOutputStream(socket.outputStream)

                val deviceId = UUID.randomUUID().toString().replace("-", "")
                val identity = """
                    {"id":0,"type":"kdeconnect.identity","body":{
                    "deviceId":"$deviceId",
                    "deviceName":"PhoneBridge Android",
                    "deviceType":"phone",
                    "incomingCapabilities":["kdeconnect.ping"],
                    "outgoingCapabilities":["kdeconnect.ping"],
                    "protocolVersion":8,
                    "certificateFingerprint":"android-test"
                    }}
                """.trimIndent().replace("\n", "")

                writePacket(output, identity)
                val pcIdentity = readPacket(input)
                    ?: return@withContext PairingResult.Failed("PC closed before identity")
                if (!pcIdentity.contains("kdeconnect.identity")) {
                    return@withContext PairingResult.Failed("Unexpected PC packet")
                }

                val pair = """
                    {"id":1,"type":"kdeconnect.pair","body":{"pair":true}}
                """.trimIndent().replace("\n", "")
                writePacket(output, pair)

                val response = readPacket(input)
                    ?: return@withContext PairingResult.Failed("PC closed before pairing response")
                if (response.contains("\"pair\":true")) {
                    PairingResult.Accepted
                } else {
                    PairingResult.Rejected
                }
            }
        } catch (e: Exception) {
            PairingResult.Failed(e.message ?: e.javaClass.simpleName)
        }
    }

    /**
     * Temporary test-only trust policy. The PC certificate is self-signed,
     * so the first integration test deliberately accepts it. Real fingerprint
     * verification must replace this before production pairing is enabled.
     */
    private fun createTestTlsContext(): SSLContext {
        val trustAll = object : X509TrustManager {
            override fun getAcceptedIssuers(): Array<X509Certificate> = emptyArray()
            override fun checkClientTrusted(chain: Array<X509Certificate>, authType: String) = Unit
            override fun checkServerTrusted(chain: Array<X509Certificate>, authType: String) = Unit
        }
        return SSLContext.getInstance("TLS").apply {
            init(null, arrayOf<TrustManager>(trustAll), SecureRandom())
        }
    }

    private fun writePacket(output: BufferedOutputStream, json: String) {
        val payload = json.toByteArray(Charsets.UTF_8)
        val length = payload.size
        output.write(byteArrayOf(
            (length ushr 24).toByte(),
            (length ushr 16).toByte(),
            (length ushr 8).toByte(),
            length.toByte(),
        ))
        output.write(payload)
        output.flush()
    }

    private fun readPacket(input: BufferedInputStream): String? {
        val header = ByteArray(4)
        readFully(input, header)
        val length = ((header[0].toInt() and 0xff) shl 24) or
            ((header[1].toInt() and 0xff) shl 16) or
            ((header[2].toInt() and 0xff) shl 8) or
            (header[3].toInt() and 0xff)
        if (length <= 0 || length > 4 * 1024 * 1024) return null
        val payload = ByteArray(length)
        readFully(input, payload)
        return payload.toString(Charsets.UTF_8)
    }

    private fun readFully(input: BufferedInputStream, buffer: ByteArray) {
        var offset = 0
        while (offset < buffer.size) {
            val count = input.read(buffer, offset, buffer.size - offset)
            if (count < 0) throw java.io.EOFException("Unexpected end of stream")
            offset += count
        }
    }
}

sealed interface PairingResult {
    data object Accepted : PairingResult
    data object Rejected : PairingResult
    data class Failed(val reason: String) : PairingResult
}
