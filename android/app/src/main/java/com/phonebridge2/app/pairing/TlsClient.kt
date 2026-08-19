package com.phonebridge2.app.pairing

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.io.BufferedReader
import java.io.BufferedWriter
import java.io.InputStreamReader
import java.io.OutputStreamWriter
import java.net.InetSocketAddress
import java.net.Socket
import java.security.MessageDigest
import javax.net.ssl.SSLContext
import javax.net.ssl.SSLSocket
import javax.net.ssl.TrustManager
import javax.net.ssl.X509TrustManager

/** TLS client for the PhoneBridge control channel. */
class TlsClient(
    private val connectTimeoutMs: Int = 5000,
    private val readTimeoutMs: Int = 15000,
) {
    data class Connection(
        val socket: SSLSocket,
        val channel: FramedChannel,
    ) {
        fun close() = socket.close()
    }

    suspend fun connect(
        host: String,
        port: Int,
        expectedServerFingerprint: String,
        hello: Message,
    ): Connection = withContext(Dispatchers.IO) {
        val sslSocket = createPinnedSocket(host, port, expectedServerFingerprint)
        sslSocket.soTimeout = readTimeoutMs
        sslSocket.startHandshake()

        val reader = BufferedReader(InputStreamReader(sslSocket.inputStream, Charsets.UTF_8))
        val writer = BufferedWriter(OutputStreamWriter(sslSocket.outputStream, Charsets.UTF_8))
        val connection = Connection(sslSocket, FramedChannel(reader, writer))
        connection.channel.write(hello)
        connection
    }

    suspend fun readMessage(connection: Connection): Message =
        connection.channel.readAsync()

    suspend fun sendMessage(connection: Connection, message: Message) =
        connection.channel.writeAsync(message)

    private fun createPinnedSocket(host: String, port: Int, expectedFingerprint: String): SSLSocket {
        val trustManager = object : X509TrustManager {
            override fun getAcceptedIssuers(): Array<java.security.cert.X509Certificate> = emptyArray()
            override fun checkClientTrusted(chain: Array<java.security.cert.X509Certificate>, authType: String) = Unit
            override fun checkServerTrusted(chain: Array<java.security.cert.X509Certificate>, authType: String) {
                require(chain.isNotEmpty()) { "PC sent no TLS certificate" }
                val actual = sha256(chain[0].encoded)
                require(normalize(actual) == normalize(expectedFingerprint)) {
                    "PhoneBridge PC certificate fingerprint mismatch"
                }
            }
        }

        val context = SSLContext.getInstance("TLS").apply {
            init(null, arrayOf<TrustManager>(trustManager), null)
        }

        val raw = Socket()
        raw.connect(InetSocketAddress(host, port), connectTimeoutMs)
        return context.socketFactory.createSocket(raw, host, port, true) as SSLSocket
    }

    companion object {
        fun sha256(bytes: ByteArray): String = MessageDigest.getInstance("SHA-256")
            .digest(bytes)
            .joinToString(":") { "%02X".format(it) }

        private fun normalize(value: String) = value.replace(":", "").replace(" ", "").uppercase()
    }
}
