package com.phonebridge.app.service

import android.app.*
import android.content.Intent
import android.media.AudioAttributes
import android.media.AudioFormat
import android.media.AudioPlaybackCaptureConfiguration
import android.media.AudioRecord
import android.media.projection.MediaProjection
import android.media.projection.MediaProjectionManager
import android.os.Build
import android.os.IBinder
import androidx.core.app.NotificationCompat
import com.phonebridge.app.MainActivity
import com.phonebridge.app.R
import com.phonebridge.app.opus.OpusEncoder
import java.net.DatagramPacket
import java.net.DatagramSocket
import java.net.InetAddress
import java.nio.ByteBuffer
import kotlin.concurrent.thread

class AudioCaptureService : Service() {

    private var mediaProjection: MediaProjection? = null
    private var audioRecord: AudioRecord? = null
    private var udpSocket: DatagramSocket? = null
    private var isCapturing = false
    private var sequenceNumber: Short = 0
    private val opusEncoder = OpusEncoder()

    companion object {
        const val CHANNEL_ID = "phonebridge_audio"
        const val NOTIFICATION_ID = 1
        const val SAMPLE_RATE = 48000
        const val CHANNEL_CONFIG = AudioFormat.CHANNEL_IN_MONO
        const val AUDIO_FORMAT = AudioFormat.ENCODING_PCM_16BIT
        const val BUFFER_SIZE = 1920
    }

    override fun onCreate() {
        super.onCreate()
        createNotificationChannel()
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        val code = intent?.getIntExtra("code", -1) ?: -1
        val data = intent?.getParcelableExtra<Intent>("data")

        if (code == -1 || data == null) {
            stopSelf()
            return START_NOT_STICKY
        }

        val manager = getSystemService(MEDIA_PROJECTION_SERVICE) as MediaProjectionManager
        mediaProjection = manager.getMediaProjection(code, data)

        startForeground(NOTIFICATION_ID, buildNotification())
        startCapture()

        return START_STICKY
    }

    private fun startCapture() {
        val config = AudioPlaybackCaptureConfiguration.Builder(mediaProjection!!)
            .addMatchingUsage(AudioAttributes.USAGE_MEDIA)
            .addMatchingUsage(AudioAttributes.USAGE_GAME)
            .addMatchingUsage(AudioAttributes.USAGE_UNKNOWN)
            .build()

        val minBuffer = AudioRecord.getMinBufferSize(SAMPLE_RATE, CHANNEL_CONFIG, AUDIO_FORMAT)
        audioRecord = AudioRecord.Builder()
            .setAudioFormat(
                AudioFormat.Builder()
                    .setEncoding(AUDIO_FORMAT)
                    .setSampleRate(SAMPLE_RATE)
                    .setChannelMask(CHANNEL_CONFIG)
                    .build()
            )
            .setBufferSizeInBytes(minBuffer.coerceAtLeast(BUFFER_SIZE * 2))
            .setAudioPlaybackCaptureConfig(config)
            .build()

        udpSocket = DatagramSocket()

        audioRecord?.startRecording()
        isCapturing = true

        thread(name = "AudioCapture") {
            val pcmBuffer = ShortArray(960)
            val opusBuffer = ByteArray(1500)
            val packetBuffer = ByteBuffer.allocate(1500)
            val pcAddress = detectGatewayIp()

            while (isCapturing) {
                val read = audioRecord?.read(pcmBuffer, 0, pcmBuffer.size) ?: 0
                if (read > 0) {
                    val encoded = opusEncoder.encode(pcmBuffer, opusBuffer)
                    if (encoded > 0) {
                        packetBuffer.clear()
                        packetBuffer.putShort(sequenceNumber++)
                        packetBuffer.put(opusBuffer, 0, encoded)

                        val packet = DatagramPacket(
                            packetBuffer.array(),
                            packetBuffer.position(),
                            pcAddress,
                            5001
                        )
                        try {
                            udpSocket?.send(packet)
                        } catch (e: Exception) {
                            // PC not reachable yet
                        }
                    }
                }
            }
        }
    }

    private fun detectGatewayIp(): InetAddress {
        return try {
            val wm = getSystemService(WIFI_SERVICE) as android.net.wifi.WifiManager
            val dhcp = wm.dhcpInfo
            val gateway = dhcp.gateway
            val ip = String.format(
                "%d.%d.%d.%d",
                gateway and 0xFF,
                gateway shr 8 and 0xFF,
                gateway shr 16 and 0xFF,
                gateway shr 24 and 0xFF
            )
            InetAddress.getByName(ip)
        } catch (e: Exception) {
            InetAddress.getByName("192.168.137.1")
        }
    }

    override fun onDestroy() {
        isCapturing = false
        audioRecord?.stop()
        audioRecord?.release()
        mediaProjection?.stop()
        udpSocket?.close()
        opusEncoder.destroy()
        super.onDestroy()
    }

    override fun onBind(intent: Intent?): IBinder? = null

    private fun createNotificationChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = NotificationChannel(
                CHANNEL_ID,
                "PhoneBridge Audio Capture",
                NotificationManager.IMPORTANCE_LOW
            )
            val manager = getSystemService(NotificationManager::class.java)
            manager.createNotificationChannel(channel)
        }
    }

    private fun buildNotification(): Notification {
        val pendingIntent = PendingIntent.getActivity(
            this, 0,
            Intent(this, MainActivity::class.java),
            PendingIntent.FLAG_IMMUTABLE
        )
        return NotificationCompat.Builder(this, CHANNEL_ID)
            .setContentTitle("PhoneBridge")
            .setContentText("Streaming audio to PC...")
            .setSmallIcon(R.drawable.ic_notification)
            .setContentIntent(pendingIntent)
            .setOngoing(true)
            .build()
    }
}
