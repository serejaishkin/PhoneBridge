package com.phonebridge.app

import android.Manifest
import android.content.Intent
import android.media.projection.MediaProjectionManager
import android.os.Bundle
import android.provider.Settings
import android.widget.Toast
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.phonebridge.app.call.CallManager
import com.phonebridge.app.discovery.BleAdvertiser
import com.phonebridge.app.media.MediaControllerBridge
import com.phonebridge.app.service.AudioCaptureService
import com.phonebridge.app.sms.SmsBridge
import com.phonebridge.app.ui.theme.PhoneBridgeTheme

class MainActivity : ComponentActivity() {
    private val bleAdvertiser by lazy { BleAdvertiser(this) }
    private val callManager by lazy { CallManager(this) }

    private var pendingHost = "192.168.137.1"

    private val permissionsLauncher = registerForActivityResult(
        ActivityResultContracts.RequestMultiplePermissions()
    ) { permissions ->
        if (permissions.entries.all { it.value }) startBridge()
        else Toast.makeText(this, "Нужны разрешения для звонков и SMS", Toast.LENGTH_LONG).show()
    }

    /** System dialog asking the user which audio to share; result starts capture. */
    private val mediaProjectionLauncher = registerForActivityResult(
        ActivityResultContracts.StartActivityForResult()
    ) { result ->
        if (result.resultCode == RESULT_OK && result.data != null) {
            startAudioCapture(result.resultCode, result.data!!)
        } else {
            Toast.makeText(this, "Трансляция звука не разрешена", Toast.LENGTH_SHORT).show()
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        SmsBridge.init(this)
        setContent {
            PhoneBridgeTheme {
                Surface(Modifier.fillMaxSize(), color = MaterialTheme.colorScheme.background) {
                    MainScreen(
                        onStart = { host -> requestPermissions(host) },
                        onEnableMediaAccess = { startActivity(Intent(Settings.ACTION_NOTIFICATION_LISTENER_SETTINGS)) },
                        onCastAudio = { requestAudioCaptureConsent() },
                        onStop = { stopBridge() }
                    )
                }
            }
        }
    }

    private fun requestPermissions(host: String) {
        pendingHost = host.trim().ifBlank { "192.168.137.1" }
        permissionsLauncher.launch(arrayOf(
            Manifest.permission.INTERNET,
            Manifest.permission.ACCESS_WIFI_STATE,
            Manifest.permission.ACCESS_NETWORK_STATE,
            Manifest.permission.READ_PHONE_STATE,
            Manifest.permission.ANSWER_PHONE_CALLS,
            Manifest.permission.READ_SMS,
            Manifest.permission.RECEIVE_SMS,
            Manifest.permission.SEND_SMS
        ))
    }

    private fun requestAudioCaptureConsent() {
        val manager = getSystemService(MEDIA_PROJECTION_SERVICE) as MediaProjectionManager
        mediaProjectionLauncher.launch(manager.createScreenCaptureIntent())
    }

    private fun startAudioCapture(resultCode: Int, data: Intent) {
        val serviceIntent = Intent(this, AudioCaptureService::class.java).apply {
            putExtra(AudioCaptureService.EXTRA_CODE, resultCode)
            putExtra(AudioCaptureService.EXTRA_DATA, data)
            putExtra(AudioCaptureService.EXTRA_HOST, pendingHost)
        }
        startForegroundService(serviceIntent)
        Toast.makeText(this, "Звук транслируется на $pendingHost", Toast.LENGTH_SHORT).show()
    }

    private fun startBridge() {
        callManager.start(pendingHost)
        MediaControllerBridge.init(this)
        SmsBridge.init(this)
        bleAdvertiser.start()
        Toast.makeText(this, "PhoneBridge подключается к $pendingHost", Toast.LENGTH_SHORT).show()
    }

    private fun stopBridge() {
        callManager.stop()
        bleAdvertiser.stop()
        stopService(Intent(this, AudioCaptureService::class.java))
        Toast.makeText(this, "PhoneBridge остановлен", Toast.LENGTH_SHORT).show()
    }

    override fun onDestroy() {
        callManager.stop()
        bleAdvertiser.stop()
        super.onDestroy()
    }
}

@Composable
private fun MainScreen(
    onStart: (String) -> Unit,
    onEnableMediaAccess: () -> Unit,
    onCastAudio: () -> Unit,
    onStop: () -> Unit
) {
    var host by remember { mutableStateOf("192.168.137.1") }
    Column(
        Modifier.fillMaxSize().padding(24.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center
    ) {
        Text("PhoneBridge", style = MaterialTheme.typography.headlineLarge)
        Spacer(Modifier.height(8.dp))
        Text("Звонки • Медиа • SMS", style = MaterialTheme.typography.bodyMedium)
        Spacer(Modifier.height(24.dp))
        OutlinedTextField(
            value = host,
            onValueChange = { host = it },
            label = { Text("IP компьютера") },
            singleLine = true
        )
        Spacer(Modifier.height(12.dp))
        Button(onClick = { onStart(host) }) { Text("Подключить") }
        Spacer(Modifier.height(12.dp))
        Button(onClick = onCastAudio) { Text("Трансляция звука на ПК") }
        Spacer(Modifier.height(12.dp))
        OutlinedButton(onClick = onEnableMediaAccess) { Text("Доступ к медиа") }
        Spacer(Modifier.height(12.dp))
        OutlinedButton(onClick = onStop) { Text("Остановить") }
    }
}
