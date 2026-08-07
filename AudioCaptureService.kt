package com.phonebridge.app

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Context
import android.content.Intent
import android.media.AudioAttributes
import android.media.AudioFormat
import android.media.AudioPlaybackCaptureConfiguration
import android.media.AudioRecord
import android.media.MediaCodec
import android.media.MediaFormat
import android.media.projection.MediaProjection
import android.media.projection.MediaProjectionManager
import android.os.Build
import android.os.IBinder
import android.util.Log
import androidx.core.app.NotificationCompat
import kotlinx.coroutines.*
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.WebSocket
import okhttp3.WebSocketListener
import java.net.DatagramPacket
import java.net.DatagramSocket
import java.net.InetAddress

class AudioCaptureService : Service() {
    private val TAG = "PhoneBridge"
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO)
    private var mediaProjection: MediaProjection? = null
    private var audioRecord: AudioRecord? = null
    private var codec: MediaCodec? = null
    private var udpSocket: DatagramSocket? = null
    private var webSocket: WebSocket? = null

    private val sampleRate = 48000
    private val channels = 2
    private val bitrate = 64000
    private var sequenceNumber = 0

    override fun onCreate() {
        super.onCreate()
        createNotificationChannel()
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        val code = intent?.getIntExtra("code", -1) ?: -1
        @Suppress("DEPRECATION")
        val data = intent?.getParcelableExtra<Intent>("data")
        val pcIp = intent?.getStringExtra("pc_ip") ?: "192.168.137.1"

        if (code == -1 || data == null) {
            stopSelf()
            return START_NOT_STICKY
        }

        startForeground(1, buildNotification())

        val mgr = getSystem
