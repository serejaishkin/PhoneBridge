package com.phonebridge.app.pairing

import android.content.Context
import android.util.Base64
import java.io.File
import java.math.BigInteger
import java.security.KeyPairGenerator
import java.security.MessageDigest
import java.security.SecureRandom
import java.security.cert.X509Certificate
import java.util.Date
import javax.security.auth.x500.X500Principal
import org.bouncycastle.cert.jcajce.JcaX509CertificateConverter
import org.bouncycastle.cert.jcajce.JcaX509v3CertificateBuilder
import org.bouncycastle.operator.jcajce.JcaContentSignerBuilder

/**
 * Phone-side persistent identity: a self-signed certificate stored for years,
 * mirroring pc/src/pairing/identity.rs. Trust between devices is based on the
 * SHA-256 certificate fingerprint, not on a certificate authority.
 *
 * Generation performs RSA keygen and must run off the main thread; CallManager
 * evaluates this lazily inside the connection worker thread.
 */
class PhoneIdentity private constructor(
    val deviceId: String,
    val certDer: ByteArray,
    @Suppress("unused") private val certPem: String,
    @Suppress("unused") private val privateKeyPem: String
) {
    /** SHA-256 hex fingerprint of the DER-encoded certificate. */
    fun fingerprintHex(): String {
        val digest = MessageDigest.getInstance("SHA-256").digest(certDer)
        return digest.joinToString("") { "%02x".format(it) }
    }

    companion object {
        private const val CERT_FILE = "cert.pem"
        private const val KEY_FILE = "key.pem"
        private const val ID_FILE = "device_id.txt"

        fun loadOrCreate(context: Context): PhoneIdentity {
            val dir = File(context.filesDir, "identity").apply { mkdirs() }
            val certFile = File(dir, CERT_FILE)
            val keyFile = File(dir, KEY_FILE)
            val idFile = File(dir, ID_FILE)

            if (certFile.exists() && keyFile.exists() && idFile.exists()) {
                val certPem = certFile.readText()
                return PhoneIdentity(idFile.readText().trim(), pemToDer(certPem), certPem, keyFile.readText())
            }

            // Same id format as the PC side ("pb2-" + hex).
            val deviceId = "pb2-" + randomHex(16)
            val keyPair = KeyPairGenerator.getInstance("RSA").apply { initialize(2048) }.generateKeyPair()

            val now = Date()
            val expiry = Date(now.time + 20L * 365 * 24 * 3600 * 1000)
            val serial = BigInteger(64, SecureRandom())
            val subject = X500Principal("CN=$deviceId")

            val certBuilder = JcaX509v3CertificateBuilder(subject, serial, now, expiry, subject, keyPair.public)
            val signer = JcaContentSignerBuilder("SHA256withRSA").build(keyPair.private)
            val cert: X509Certificate = JcaX509CertificateConverter().getCertificate(certBuilder.build(signer))

            val certDer = cert.encoded
            val certPem = derToPem(certDer, "CERTIFICATE")
            val keyPem = derToPem(keyPair.private.encoded, "PRIVATE KEY")

            certFile.writeText(certPem)
            keyFile.writeText(keyPem)
            idFile.writeText(deviceId)

            return PhoneIdentity(deviceId, certDer, certPem, keyPem)
        }

        private fun derToPem(der: ByteArray, label: String): String {
            val b64 = Base64.encodeToString(der, Base64.NO_WRAP)
            val chunks = b64.chunked(64).joinToString("\n")
            return "-----BEGIN $label-----\n$chunks\n-----END $label-----\n"
        }

        private fun pemToDer(pem: String): ByteArray {
            val b64 = pem.lines().filterNot { it.startsWith("-----") }.joinToString("")
            return Base64.decode(b64, Base64.DEFAULT)
        }

        private fun randomHex(bytes: Int): String {
            val b = ByteArray(bytes)
            SecureRandom().nextBytes(b)
            return b.joinToString("") { "%02x".format(it) }
        }
    }
}
