package com.phonebridge.app.service

import android.app.*
import android.content.Intent
import android.media.AudioAttributes
import android.media.AudioFormat
import android.media.AudioTrack
import android.os.Build
import android.os.IBinder
import androidx.core.app.NotificationCompat
import com.phonebridge.app.MainActivity
import com.phonebridge.app.R
import com.phonebridge.app.opus.OpusDecoder
import java.net.DatagramPacket
import java.net.DatagramSocket
import kotlin.concurrent.thread

class AudioPlaybackService : Service() {

    private var udpSocket: DatagramSocket? = null
    private var audioTrack: AudioTrack? = null
    private var isPlaying = false
    private val opusDecoder = OpusDecoder()

    companion object {
        const val CHANNEL_ID = "phonebridge_playback"
        const val NOTIFICATION_ID = 2
        const val SAMPLE_RATE = 48000
        const val BUFFER_SIZE = 1920
    }

    override fun onCreate() {
        super.onCreate()
        createNotificationChannel()
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        startForeground(NOTIFICATION_ID, buildNotification())
        startPlayback()
        return START_STICKY
    }

    private fun startPlayback() {
        val minBuffer = AudioTrack.getMinBufferSize(
            SAMPLE_RATE,
            AudioFormat.CHANNEL_OUT_MONO,
            AudioFormat.ENCODING_PCM_16BIT
        )

        audioTrack = AudioTrack.Builder()
            .setAudioAttributes(
                AudioAttributes.Builder()
                    .setUsage(AudioAttributes.USAGE_VOICE_COMMUNICATION)
                    .setContentType(AudioAttributes.CONTENT_TYPE_SPEECH)
                    .build()
            )
            .setAudioFormat(
                AudioFormat.Builder()
                    .setSampleRate(SAMPLE_RATE)
                    .setChannelMask(AudioFormat.CHANNEL_OUT_MONO)
                    .setEncoding(AudioFormat.ENCODING_PCM_16BIT)
                    .build()
            )
            .setBufferSizeInBytes(minBuffer.coerceAtLeast(BUFFER_SIZE * 4))
            .setTransferMode(AudioTrack.MODE_STREAM)
            .build()

        audioTrack?.play()
        udpSocket = DatagramSocket(5003)
        isPlaying = true

        thread(name = "AudioPlayback") {
            val packetBuf = ByteArray(1500)
            val pcmBuf = ShortArray(960)

            while (isPlaying) {
                try {
                    val packet = DatagramPacket(packetBuf, packetBuf.size)
                    udpSocket?.receive(packet)

                    if (packet.length < 2) continue
                    // seq = packet.data[0..1], opus = packet.data[2..]
                    val opusData = packet.data.copyOfRange(2, packet.length)

                    val decoded = opusDecoder.decode(opusData, pcmBuf)
                    if (decoded > 0) {
                        audioTrack?.write(pcmBuf, 0, decoded)
                    }
                } catch (e: Exception) {
                    // ignore
                }
            }
        }
    }

    override fun onDestroy() {
        isPlaying = false
        audioTrack?.stop()
        audioTrack?.release()
        udpSocket?.close()
        opusDecoder.destroy()
        super.onDestroy()
    }

    override fun onBind(intent: Intent?): IBinder? = null

    private fun createNotificationChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = NotificationChannel(
                CHANNEL_ID,
                "PhoneBridge Mic Playback",
                NotificationManager.IMPORTANCE_LOW
            )
            getSystemService(NotificationManager::class.java)?.createNotificationChannel(channel)
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
            .setContentText("Receiving PC microphone...")
            .setSmallIcon(R.drawable.ic_notification)
            .setContentIntent(pendingIntent)
            .setOngoing(true)
            .build()
    }
}
