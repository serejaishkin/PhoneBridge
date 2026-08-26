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

/**
 * TLS + newline-delimited JSON control client matching pc/src/protocol.rs.
 *
 * Security model (mutual TLS):
 * - The phone presents its persistent certificate during the handshake itself;
 *   Hello additionally carries the fingerprint so the PC can cross-check that
 *   the claimed device id belongs to the authenticated certificate.
 * - The server certificate is pinned per host on first connect (TOFU); later
 *   connections require an exact match. The HelloAck fingerprint is cross-checked
 *   against the pinned value to defend against a different machine reusing the IP.
 */
class SignalingClient(
    private val onCommand: (String, Map<String, String>) -> Unit = { _, _ -> },
    private val onStatus: (String, String?) -> Unit = { _, _ -> },
    private val identityProvider: () -> PhoneIdentity? = { null },
    private val trustProvider: () -> TrustStore? = { null }
) {
