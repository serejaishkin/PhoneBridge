package com.phonebridge.app

import android.Manifest
import android.content.Intent
import android.media.projection.MediaProjectionManager
import android.os.Build
import android.os.Bundle
import android.provider.Settings
import android.widget.Toast
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.phonebridge.app.call.CallManager
import com.phonebridge.app.discovery.BleAdvertiser
import com.phonebridge.app.media.MediaControllerBridge
import com.phonebridge.app.service.AudioCaptureService
import com.phonebridge.app.service.AudioPlaybackService
import com.phonebridge.app.ui.theme.PhoneBridgeTheme

class MainActivity : ComponentActivity() {

    private val bleAdvertiser by lazy { BleAdvertiser(this) }
    private val callManager by lazy { CallManager(this) }

    private val projectionLauncher = registerForActivityResult(
        ActivityResultContracts.StartActivityForResult()
    ) { result ->
        if (result.resultCode == RESULT_OK && result.data != null) {
            val intent = Intent(this, AudioCaptureService::class.java).apply {
                putExtra("code", result.resultCode)
                putExtra("data", result.data)
            }
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                startForegroundService(intent)
            } else {
                startService(intent)
            }
            Toast.makeText(this, "Audio capture started", Toast.LENGTH_SHORT).show()
        }
    }

    private val permissionsLauncher = registerForActivityResult(
        ActivityResultContracts.RequestMultiplePermissions()
    ) { permissions ->
        val allGranted = permissions.entries.all { it.value }
        if (allGranted) {
            startMediaProjection()
            startServices()
        } else {
            Toast.makeText(this, "Permissions required", Toast.LENGTH_LONG).show()
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            PhoneBridgeTheme {
                Surface(
                    modifier = Modifier.fillMaxSize(),
                    color = MaterialTheme.colorScheme.background
                ) {
                    MainScreen(
                        onStartCapture = { requestPermissions() },
                        onEnableMediaAccess = {
                            startActivity(Intent(Settings.ACTION_NOTIFICATION_LISTENER_SETTINGS))
                        },
                        onStopCapture = {
                            stopService(Intent(this, AudioCaptureService::class.java))
                            stopService(Intent(this, AudioPlaybackService::class.java))
                            callManager.stop()
                            bleAdvertiser.stop()
                        }
                    )
                }
            }
        }
    }

    private fun requestPermissions() {
        val permissions = mutableListOf(
            Manifest.permission.RECORD_AUDIO,
            Manifest.permission.INTERNET,
            Manifest.permission.ACCESS_WIFI_STATE,
            Manifest.permission.ACCESS_NETWORK_STATE,
            Manifest.permission.READ_PHONE_STATE,
            Manifest.permission.ANSWER_PHONE_CALLS,
            Manifest.permission.BLUETOOTH,
            Manifest.permission.BLUETOOTH_ADMIN,
        )
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
            permissions.add(Manifest.permission.BLUETOOTH_ADVERTISE)
            permissions.add(Manifest.permission.BLUETOOTH_CONNECT)
            permissions.add(Manifest.permission.BLUETOOTH_SCAN)
        }
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            permissions.add(Manifest.permission.POST_NOTIFICATIONS)
        }
        permissionsLauncher.launch(permissions.toTypedArray())
    }

    private fun startServices() {
        // One signaling connection carries both call events and media commands.
        callManager.start()
        MediaControllerBridge.init(this)

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            startForegroundService(Intent(this, AudioPlaybackService::class.java))
        } else {
            startService(Intent(this, AudioPlaybackService::class.java))
        }

        bleAdvertiser.start()
    }

    private fun startMediaProjection() {
        val manager = getSystemService(MEDIA_PROJECTION_SERVICE) as MediaProjectionManager
        projectionLauncher.launch(manager.createScreenCaptureIntent())
    }

    override fun onDestroy() {
        callManager.stop()
        bleAdvertiser.stop()
        super.onDestroy()
    }
}

@Composable
fun MainScreen(
    onStartCapture: () -> Unit,
    onEnableMediaAccess: () -> Unit,
    onStopCapture: () -> Unit
) {
    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(24.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center
    ) {
        Text(
            text = "PhoneBridge",
            style = MaterialTheme.typography.headlineLarge
        )
        Spacer(modifier = Modifier.height(8.dp))
        Text(
            text = "Call and media control bridge",
            style = MaterialTheme.typography.bodyMedium
        )
        Spacer(modifier = Modifier.height(32.dp))
        Button(onClick = onStartCapture) {
            Text("Start Bridge")
        }
        Spacer(modifier = Modifier.height(12.dp))
        OutlinedButton(onClick = onEnableMediaAccess) {
            Text("Enable media access")
        }
        Spacer(modifier = Modifier.height(12.dp))
        OutlinedButton(onClick = onStopCapture) {
            Text("Stop")
        }
    }
}
